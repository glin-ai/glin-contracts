// Copyright (C) 2025 GLIN Team
// SPDX-License-Identifier: Apache-2.0
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod generic_escrow {
    use ink::storage::Mapping;

    /// Milestone status
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum MilestoneStatus {
        Pending,
        Completed,
        Disputed,
        Resolved,
        Cancelled,
    }

    /// Milestone definition
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Milestone {
        pub description: ink::prelude::string::String,
        pub amount: Balance,
        pub status: MilestoneStatus,
        pub deadline: Timestamp,
        pub oracle_verification: bool,
    }

    /// Escrow agreement
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Agreement {
        pub client: AccountId,
        pub provider: AccountId,
        pub total_amount: Balance,
        pub deposited_amount: Balance,
        pub created_at: Timestamp,
        pub dispute_timeout: Timestamp,
        pub oracle: Option<AccountId>,
        pub is_active: bool,
    }

    /// The generic escrow contract storage
    #[ink(storage)]
    pub struct GenericEscrow {
        /// Next agreement ID
        next_agreement_id: u128,
        /// Mapping from agreement ID to agreement
        agreements: Mapping<u128, Agreement>,
        /// Mapping from (agreement_id, milestone_index) to milestone
        milestones: Mapping<(u128, u32), Milestone>,
        /// Mapping from agreement ID to number of milestones
        milestone_counts: Mapping<u128, u32>,
        /// Platform fee percentage (in basis points, 100 = 1%)
        platform_fee_bps: u16,
        /// Platform fee recipient
        platform_account: AccountId,
    }

    /// Events
    #[ink(event)]
    pub struct AgreementCreated {
        #[ink(topic)]
        agreement_id: u128,
        #[ink(topic)]
        client: AccountId,
        #[ink(topic)]
        provider: AccountId,
        total_amount: Balance,
    }

    #[ink(event)]
    pub struct MilestoneCompleted {
        #[ink(topic)]
        agreement_id: u128,
        milestone_index: u32,
        amount: Balance,
    }

    #[ink(event)]
    pub struct DisputeRaised {
        #[ink(topic)]
        agreement_id: u128,
        milestone_index: u32,
        raised_by: AccountId,
    }

    #[ink(event)]
    pub struct FundsReleased {
        #[ink(topic)]
        agreement_id: u128,
        #[ink(topic)]
        to: AccountId,
        amount: Balance,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        AgreementNotFound,
        MilestoneNotFound,
        NotAuthorized,
        InsufficientFunds,
        InvalidMilestoneStatus,
        AgreementNotActive,
        MilestoneAlreadyCompleted,
        DisputeTimeoutNotReached,
        TransferFailed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl GenericEscrow {
        /// Constructor
        #[ink(constructor)]
        pub fn new(platform_account: AccountId, platform_fee_bps: u16) -> Self {
            Self {
                next_agreement_id: 0,
                agreements: Mapping::default(),
                milestones: Mapping::default(),
                milestone_counts: Mapping::default(),
                platform_fee_bps,
                platform_account,
            }
        }

        /// Create a new escrow agreement
        #[ink(message, payable)]
        pub fn create_agreement(
            &mut self,
            provider: AccountId,
            milestone_descriptions: ink::prelude::vec::Vec<ink::prelude::string::String>,
            milestone_amounts: ink::prelude::vec::Vec<Balance>,
            milestone_deadlines: ink::prelude::vec::Vec<Timestamp>,
            dispute_timeout: Timestamp,
            oracle: Option<AccountId>,
        ) -> Result<u128> {
            let caller = self.env().caller();
            let transferred = self.env().transferred_value();

            // Validate inputs
            let milestone_count = milestone_descriptions.len();
            if milestone_count != milestone_amounts.len()
                || milestone_count != milestone_deadlines.len()
                || milestone_count == 0 {
                return Err(Error::InvalidMilestoneStatus);
            }

            let total_amount: Balance = milestone_amounts.iter().sum();
            if transferred < total_amount {
                return Err(Error::InsufficientFunds);
            }

            let agreement_id = self.next_agreement_id;
            self.next_agreement_id = self.next_agreement_id
                .checked_add(1)
                .expect("Agreement ID overflow");

            // Create agreement
            let agreement = Agreement {
                client: caller,
                provider,
                total_amount,
                deposited_amount: transferred,
                created_at: self.env().block_timestamp(),
                dispute_timeout,
                oracle,
                is_active: true,
            };

            self.agreements.insert(agreement_id, &agreement);

            // Create milestones
            for (i, (desc, amount, deadline)) in milestone_descriptions
                .iter()
                .zip(milestone_amounts.iter())
                .zip(milestone_deadlines.iter())
                .map(|((d, a), dl)| (d, a, dl))
                .enumerate()
            {
                let milestone = Milestone {
                    description: desc.clone(),
                    amount: *amount,
                    status: MilestoneStatus::Pending,
                    deadline: *deadline,
                    oracle_verification: oracle.is_some(),
                };

                let milestone_index = u32::try_from(i).expect("Too many milestones");
                self.milestones.insert((agreement_id, milestone_index), &milestone);
            }

            let milestone_count_u32 = u32::try_from(milestone_count).expect("Milestone count overflow");
            self.milestone_counts.insert(agreement_id, &milestone_count_u32);

            self.env().emit_event(AgreementCreated {
                agreement_id,
                client: caller,
                provider,
                total_amount,
            });

            Ok(agreement_id)
        }

        /// Mark milestone as completed (by provider)
        #[ink(message)]
        pub fn complete_milestone(&mut self, agreement_id: u128, milestone_index: u32) -> Result<()> {
            let caller = self.env().caller();
            let agreement = self.agreements.get(agreement_id).ok_or(Error::AgreementNotFound)?;

            if caller != agreement.provider {
                return Err(Error::NotAuthorized);
            }

            if !agreement.is_active {
                return Err(Error::AgreementNotActive);
            }

            let mut milestone = self.milestones.get((agreement_id, milestone_index))
                .ok_or(Error::MilestoneNotFound)?;

            if milestone.status != MilestoneStatus::Pending {
                return Err(Error::MilestoneAlreadyCompleted);
            }

            milestone.status = MilestoneStatus::Completed;
            self.milestones.insert((agreement_id, milestone_index), &milestone);

            self.env().emit_event(MilestoneCompleted {
                agreement_id,
                milestone_index,
                amount: milestone.amount,
            });

            Ok(())
        }

        /// Approve milestone and release funds (by client or oracle)
        #[ink(message)]
        pub fn approve_and_release(
            &mut self,
            agreement_id: u128,
            milestone_index: u32,
        ) -> Result<()> {
            let caller = self.env().caller();
            let agreement = self.agreements.get(agreement_id).ok_or(Error::AgreementNotFound)?;

            // Check authorization
            let authorized = caller == agreement.client
                || agreement.oracle == Some(caller);

            if !authorized {
                return Err(Error::NotAuthorized);
            }

            if !agreement.is_active {
                return Err(Error::AgreementNotActive);
            }

            let mut milestone = self.milestones.get((agreement_id, milestone_index))
                .ok_or(Error::MilestoneNotFound)?;

            if milestone.status != MilestoneStatus::Completed {
                return Err(Error::InvalidMilestoneStatus);
            }

            milestone.status = MilestoneStatus::Resolved;
            self.milestones.insert((agreement_id, milestone_index), &milestone);

            // Calculate platform fee (checked arithmetic)
            let fee_bps = u128::from(self.platform_fee_bps);
            let platform_fee = milestone.amount
                .checked_mul(fee_bps)
                .and_then(|v| v.checked_div(10000))
                .expect("Platform fee calculation overflow");
            let provider_amount = milestone.amount
                .checked_sub(platform_fee)
                .expect("Platform fee exceeds milestone amount");

            // Transfer funds
            if platform_fee > 0 {
                self.env().transfer(self.platform_account, platform_fee)
                    .map_err(|_| Error::TransferFailed)?;
            }

            self.env().transfer(agreement.provider, provider_amount)
                .map_err(|_| Error::TransferFailed)?;

            self.env().emit_event(FundsReleased {
                agreement_id,
                to: agreement.provider,
                amount: provider_amount,
            });

            Ok(())
        }

        /// Raise a dispute
        #[ink(message)]
        pub fn raise_dispute(&mut self, agreement_id: u128, milestone_index: u32) -> Result<()> {
            let caller = self.env().caller();
            let agreement = self.agreements.get(agreement_id).ok_or(Error::AgreementNotFound)?;

            // Only client or provider can raise disputes
            if caller != agreement.client && caller != agreement.provider {
                return Err(Error::NotAuthorized);
            }

            let mut milestone = self.milestones.get((agreement_id, milestone_index))
                .ok_or(Error::MilestoneNotFound)?;

            if milestone.status != MilestoneStatus::Completed {
                return Err(Error::InvalidMilestoneStatus);
            }

            milestone.status = MilestoneStatus::Disputed;
            self.milestones.insert((agreement_id, milestone_index), &milestone);

            self.env().emit_event(DisputeRaised {
                agreement_id,
                milestone_index,
                raised_by: caller,
            });

            Ok(())
        }

        /// Resolve dispute (by oracle or timeout)
        #[ink(message)]
        pub fn resolve_dispute(
            &mut self,
            agreement_id: u128,
            milestone_index: u32,
            release_to_provider: bool,
        ) -> Result<()> {
            let caller = self.env().caller();
            let agreement = self.agreements.get(agreement_id).ok_or(Error::AgreementNotFound)?;

            // Oracle can resolve anytime, otherwise check timeout
            if agreement.oracle != Some(caller) {
                if caller != agreement.client {
                    return Err(Error::NotAuthorized);
                }
                if self.env().block_timestamp() < agreement.dispute_timeout {
                    return Err(Error::DisputeTimeoutNotReached);
                }
            }

            let mut milestone = self.milestones.get((agreement_id, milestone_index))
                .ok_or(Error::MilestoneNotFound)?;

            if milestone.status != MilestoneStatus::Disputed {
                return Err(Error::InvalidMilestoneStatus);
            }

            milestone.status = MilestoneStatus::Resolved;
            self.milestones.insert((agreement_id, milestone_index), &milestone);

            let recipient = if release_to_provider {
                agreement.provider
            } else {
                agreement.client
            };

            // Calculate fees (checked arithmetic)
            let platform_fee = if release_to_provider {
                let fee_bps = u128::from(self.platform_fee_bps);
                milestone.amount
                    .checked_mul(fee_bps)
                    .and_then(|v| v.checked_div(10000))
                    .expect("Platform fee calculation overflow")
            } else {
                0 // No fee if refunding to client
            };

            let final_amount = milestone.amount
                .checked_sub(platform_fee)
                .expect("Platform fee exceeds milestone amount");

            if platform_fee > 0 {
                self.env().transfer(self.platform_account, platform_fee)
                    .map_err(|_| Error::TransferFailed)?;
            }

            self.env().transfer(recipient, final_amount)
                .map_err(|_| Error::TransferFailed)?;

            self.env().emit_event(FundsReleased {
                agreement_id,
                to: recipient,
                amount: final_amount,
            });

            Ok(())
        }

        /// Get agreement details
        #[ink(message)]
        pub fn get_agreement(&self, agreement_id: u128) -> Option<Agreement> {
            self.agreements.get(agreement_id)
        }

        /// Get milestone details
        #[ink(message)]
        pub fn get_milestone(&self, agreement_id: u128, milestone_index: u32) -> Option<Milestone> {
            self.milestones.get((agreement_id, milestone_index))
        }

        /// Get milestone count for an agreement
        #[ink(message)]
        pub fn get_milestone_count(&self, agreement_id: u128) -> u32 {
            self.milestone_counts.get(agreement_id).unwrap_or(0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn create_agreement_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut contract = GenericEscrow::new(accounts.alice, 200); // 2% fee

            // Set caller as bob (client)
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000);

            let result = contract.create_agreement(
                accounts.charlie, // provider
                vec!["Milestone 1".into(), "Milestone 2".into()],
                vec![500, 500],
                vec![1000, 2000],
                3000, // dispute timeout
                None, // no oracle
            );

            assert!(result.is_ok());
            let agreement_id = result.unwrap();
            assert_eq!(agreement_id, 0);

            let agreement = contract.get_agreement(agreement_id).unwrap();
            assert_eq!(agreement.client, accounts.bob);
            assert_eq!(agreement.provider, accounts.charlie);
            assert_eq!(agreement.total_amount, 1000);
        }

        #[ink::test]
        fn complete_milestone_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut contract = GenericEscrow::new(accounts.alice, 200);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000);

            let agreement_id = contract.create_agreement(
                accounts.charlie,
                vec!["Milestone 1".into()],
                vec![1000],
                vec![1000],
                3000,
                None,
            ).unwrap();

            // Provider completes milestone
            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
            let result = contract.complete_milestone(agreement_id, 0);
            assert!(result.is_ok());

            let milestone = contract.get_milestone(agreement_id, 0).unwrap();
            assert_eq!(milestone.status, MilestoneStatus::Completed);
        }
    }
}
