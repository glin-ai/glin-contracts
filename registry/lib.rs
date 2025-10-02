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
mod professional_registry {
    use ink::storage::Mapping;

    /// Professional role types
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub enum ProfessionalRole {
        Lawyer,
        Doctor,
        Arbitrator,
        Notary,
        Auditor,
        ConsultantOther,
    }

    /// Professional profile
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct ProfessionalProfile {
        pub account: AccountId,
        pub role: ProfessionalRole,
        pub stake_amount: Balance,
        pub reputation_score: u32,
        pub total_jobs: u32,
        pub successful_jobs: u32,
        pub registered_at: Timestamp,
        pub is_active: bool,
        pub metadata_uri: ink::prelude::string::String,
    }

    /// Review/Rating
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout))]
    pub struct Review {
        pub reviewer: AccountId,
        pub rating: u8, // 1-5
        pub comment: ink::prelude::string::String,
        pub timestamp: Timestamp,
    }

    /// The professional registry contract storage
    #[ink(storage)]
    pub struct ProfessionalRegistry {
        /// Mapping from AccountId to Professional Profile
        professionals: Mapping<AccountId, ProfessionalProfile>,
        /// Minimum stake required per role
        min_stake: Mapping<ProfessionalRole, Balance>,
        /// Mapping from (professional, review_index) to Review
        reviews: Mapping<(AccountId, u32), Review>,
        /// Mapping from professional to review count
        review_counts: Mapping<AccountId, u32>,
        /// Slashing percentage for misbehavior (in basis points)
        slash_percentage_bps: u16,
        /// Contract owner
        owner: AccountId,
        /// Slash treasury
        slash_treasury: AccountId,
    }

    /// Events
    #[ink(event)]
    pub struct ProfessionalRegistered {
        #[ink(topic)]
        account: AccountId,
        role: ProfessionalRole,
        stake_amount: Balance,
    }

    #[ink(event)]
    pub struct StakeIncreased {
        #[ink(topic)]
        account: AccountId,
        new_stake: Balance,
    }

    #[ink(event)]
    pub struct ProfessionalSlashed {
        #[ink(topic)]
        account: AccountId,
        slash_amount: Balance,
        reason: ink::prelude::string::String,
    }

    #[ink(event)]
    pub struct ReviewSubmitted {
        #[ink(topic)]
        professional: AccountId,
        #[ink(topic)]
        reviewer: AccountId,
        rating: u8,
    }

    #[ink(event)]
    pub struct ProfessionalDeactivated {
        #[ink(topic)]
        account: AccountId,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        AlreadyRegistered,
        NotRegistered,
        InsufficientStake,
        NotAuthorized,
        InvalidRating,
        TransferFailed,
        ProfileInactive,
        InvalidMinStake,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl ProfessionalRegistry {
        /// Constructor
        #[ink(constructor)]
        pub fn new(
            owner: AccountId,
            slash_treasury: AccountId,
            slash_percentage_bps: u16,
        ) -> Self {
            let mut registry = Self {
                professionals: Mapping::default(),
                min_stake: Mapping::default(),
                reviews: Mapping::default(),
                review_counts: Mapping::default(),
                slash_percentage_bps,
                owner,
                slash_treasury,
            };

            // Set default minimum stakes (in smallest unit)
            registry.min_stake.insert(ProfessionalRole::Lawyer, &100_000_000_000_000_000_000); // 100 GLIN
            registry.min_stake.insert(ProfessionalRole::Doctor, &100_000_000_000_000_000_000);
            registry.min_stake.insert(ProfessionalRole::Arbitrator, &200_000_000_000_000_000_000); // 200 GLIN
            registry.min_stake.insert(ProfessionalRole::Notary, &50_000_000_000_000_000_000); // 50 GLIN
            registry.min_stake.insert(ProfessionalRole::Auditor, &150_000_000_000_000_000_000);
            registry.min_stake.insert(ProfessionalRole::ConsultantOther, &50_000_000_000_000_000_000);

            registry
        }

        /// Register as a professional
        #[ink(message, payable)]
        pub fn register(
            &mut self,
            role: ProfessionalRole,
            metadata_uri: ink::prelude::string::String,
        ) -> Result<()> {
            let caller = self.env().caller();
            let stake = self.env().transferred_value();

            // Check if already registered
            if self.professionals.contains(caller) {
                return Err(Error::AlreadyRegistered);
            }

            // Check minimum stake
            let min_stake = self.min_stake.get(&role).unwrap_or(0);
            if stake < min_stake {
                return Err(Error::InsufficientStake);
            }

            // Create profile
            let profile = ProfessionalProfile {
                account: caller,
                role: role.clone(),
                stake_amount: stake,
                reputation_score: 100, // Starting reputation
                total_jobs: 0,
                successful_jobs: 0,
                registered_at: self.env().block_timestamp(),
                is_active: true,
                metadata_uri,
            };

            self.professionals.insert(caller, &profile);
            self.review_counts.insert(caller, &0);

            self.env().emit_event(ProfessionalRegistered {
                account: caller,
                role,
                stake_amount: stake,
            });

            Ok(())
        }

        /// Increase stake
        #[ink(message, payable)]
        pub fn increase_stake(&mut self) -> Result<()> {
            let caller = self.env().caller();
            let additional_stake = self.env().transferred_value();

            let mut profile = self.professionals.get(caller).ok_or(Error::NotRegistered)?;

            profile.stake_amount = profile.stake_amount
                .checked_add(additional_stake)
                .expect("Stake amount overflow");
            self.professionals.insert(caller, &profile);

            self.env().emit_event(StakeIncreased {
                account: caller,
                new_stake: profile.stake_amount,
            });

            Ok(())
        }

        /// Submit a review for a professional
        #[ink(message)]
        pub fn submit_review(
            &mut self,
            professional: AccountId,
            rating: u8,
            comment: ink::prelude::string::String,
        ) -> Result<()> {
            let caller = self.env().caller();

            // Validate rating
            if !(1..=5).contains(&rating) {
                return Err(Error::InvalidRating);
            }

            // Check professional exists and is active
            let mut profile = self.professionals.get(professional).ok_or(Error::NotRegistered)?;
            if !profile.is_active {
                return Err(Error::ProfileInactive);
            }

            // Create review
            let review_index = self.review_counts.get(professional).unwrap_or(0);
            let review = Review {
                reviewer: caller,
                rating,
                comment,
                timestamp: self.env().block_timestamp(),
            };

            self.reviews.insert((professional, review_index), &review);
            let next_review_index = review_index
                .checked_add(1)
                .expect("Review index overflow");
            self.review_counts.insert(professional, &next_review_index);

            // Update reputation score (simple weighted average) with checked arithmetic
            let current_score_u32 = profile.reputation_score;
            let rating_u32 = u32::from(rating);
            let total_jobs_u32 = profile.total_jobs;

            let weighted_current = current_score_u32
                .checked_mul(total_jobs_u32)
                .expect("Reputation calculation overflow");
            let weighted_new = rating_u32
                .checked_mul(20)
                .expect("Rating multiplication overflow");
            let numerator = weighted_current
                .checked_add(weighted_new)
                .expect("Reputation sum overflow");
            let denominator = total_jobs_u32
                .checked_add(1)
                .expect("Total jobs overflow");
            let new_reputation = numerator
                .checked_div(denominator)
                .expect("Reputation division error");

            profile.reputation_score = new_reputation;
            profile.total_jobs = profile.total_jobs
                .checked_add(1)
                .expect("Total jobs increment overflow");

            if rating >= 4 {
                profile.successful_jobs = profile.successful_jobs
                    .checked_add(1)
                    .expect("Successful jobs increment overflow");
            }

            self.professionals.insert(professional, &profile);

            self.env().emit_event(ReviewSubmitted {
                professional,
                reviewer: caller,
                rating,
            });

            Ok(())
        }

        /// Slash a professional for misbehavior (only owner)
        #[ink(message)]
        pub fn slash(
            &mut self,
            professional: AccountId,
            reason: ink::prelude::string::String,
        ) -> Result<()> {
            let caller = self.env().caller();

            if caller != self.owner {
                return Err(Error::NotAuthorized);
            }

            let mut profile = self.professionals.get(professional).ok_or(Error::NotRegistered)?;

            // Calculate slash amount with checked arithmetic
            let slash_bps = u128::from(self.slash_percentage_bps);
            let slash_amount = profile.stake_amount
                .checked_mul(slash_bps)
                .and_then(|v| v.checked_div(10000))
                .expect("Slash calculation overflow");

            if slash_amount > 0 {
                profile.stake_amount = profile.stake_amount
                    .checked_sub(slash_amount)
                    .expect("Slash amount exceeds stake");

                // Transfer slashed amount to treasury
                self.env()
                    .transfer(self.slash_treasury, slash_amount)
                    .map_err(|_| Error::TransferFailed)?;

                self.env().emit_event(ProfessionalSlashed {
                    account: professional,
                    slash_amount,
                    reason,
                });
            }

            // Lower reputation
            profile.reputation_score = profile.reputation_score.saturating_sub(20);

            // Deactivate if stake falls below minimum
            let min_stake = self.min_stake.get(&profile.role).unwrap_or(0);
            if profile.stake_amount < min_stake {
                profile.is_active = false;
                self.env().emit_event(ProfessionalDeactivated {
                    account: professional,
                });
            }

            self.professionals.insert(professional, &profile);

            Ok(())
        }

        /// Withdraw stake (deactivates profile)
        #[ink(message)]
        pub fn withdraw_stake(&mut self) -> Result<()> {
            let caller = self.env().caller();
            let mut profile = self.professionals.get(caller).ok_or(Error::NotRegistered)?;

            let stake_amount = profile.stake_amount;
            profile.stake_amount = 0;
            profile.is_active = false;

            self.professionals.insert(caller, &profile);

            // Transfer stake back to professional
            self.env()
                .transfer(caller, stake_amount)
                .map_err(|_| Error::TransferFailed)?;

            self.env().emit_event(ProfessionalDeactivated {
                account: caller,
            });

            Ok(())
        }

        /// Update minimum stake for a role (only owner)
        #[ink(message)]
        pub fn set_min_stake(&mut self, role: ProfessionalRole, amount: Balance) -> Result<()> {
            let caller = self.env().caller();

            if caller != self.owner {
                return Err(Error::NotAuthorized);
            }

            if amount == 0 {
                return Err(Error::InvalidMinStake);
            }

            self.min_stake.insert(role, &amount);

            Ok(())
        }

        /// Get professional profile
        #[ink(message)]
        pub fn get_profile(&self, account: AccountId) -> Option<ProfessionalProfile> {
            self.professionals.get(account)
        }

        /// Get review
        #[ink(message)]
        pub fn get_review(&self, professional: AccountId, review_index: u32) -> Option<Review> {
            self.reviews.get((professional, review_index))
        }

        /// Get review count
        #[ink(message)]
        pub fn get_review_count(&self, professional: AccountId) -> u32 {
            self.review_counts.get(professional).unwrap_or(0)
        }

        /// Get minimum stake for role
        #[ink(message)]
        pub fn get_min_stake(&self, role: ProfessionalRole) -> Balance {
            self.min_stake.get(&role).unwrap_or(0)
        }

        /// Check if account is registered and active
        #[ink(message)]
        pub fn is_active_professional(&self, account: AccountId) -> bool {
            self.professionals
                .get(account)
                .map(|p| p.is_active)
                .unwrap_or(false)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn register_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut contract = ProfessionalRegistry::new(accounts.alice, accounts.alice, 1000);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(100_000_000_000_000_000_000);

            let result = contract.register(
                ProfessionalRole::Lawyer,
                "ipfs://metadata".into(),
            );

            assert!(result.is_ok());

            let profile = contract.get_profile(accounts.bob).unwrap();
            assert_eq!(profile.role, ProfessionalRole::Lawyer);
            assert_eq!(profile.reputation_score, 100);
            assert!(profile.is_active);
        }

        #[ink::test]
        fn submit_review_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut contract = ProfessionalRegistry::new(accounts.alice, accounts.alice, 1000);

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(100_000_000_000_000_000_000);

            contract.register(ProfessionalRole::Lawyer, "ipfs://metadata".into()).unwrap();

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);

            let result = contract.submit_review(
                accounts.bob,
                5,
                "Excellent service!".into(),
            );

            assert!(result.is_ok());
            assert_eq!(contract.get_review_count(accounts.bob), 1);
        }
    }
}
