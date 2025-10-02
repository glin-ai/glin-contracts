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
mod arbitration_dao {
    use ink::storage::Mapping;

    /// Dispute status
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum DisputeStatus {
        Open,
        Voting,
        Resolved,
        Appealed,
        Cancelled,
    }

    /// Vote choice
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum VoteChoice {
        InFavorOfClaimant,
        InFavorOfDefendant,
    }

    /// Dispute information
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Dispute {
        pub dispute_id: u128,
        pub claimant: AccountId,
        pub defendant: AccountId,
        pub description: ink::prelude::string::String,
        pub evidence_uri: ink::prelude::string::String,
        pub status: DisputeStatus,
        pub created_at: Timestamp,
        pub voting_ends_at: Timestamp,
        pub votes_for_claimant: Balance,
        pub votes_for_defendant: Balance,
        pub resolution: Option<VoteChoice>,
        pub can_appeal: bool,
    }

    /// Arbitrator information
    #[derive(Debug, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Arbitrator {
        pub account: AccountId,
        pub stake: Balance,
        pub disputes_participated: u32,
        pub disputes_resolved: u32,
        pub reputation: u32,
        pub is_active: bool,
    }

    /// The arbitration DAO contract storage
    #[ink(storage)]
    pub struct ArbitrationDAO {
        /// Next dispute ID
        next_dispute_id: u128,
        /// Mapping from dispute ID to Dispute
        disputes: Mapping<u128, Dispute>,
        /// Mapping from arbitrator account to Arbitrator info
        arbitrators: Mapping<AccountId, Arbitrator>,
        /// Mapping from (dispute_id, arbitrator) to vote
        votes: Mapping<(u128, AccountId), VoteChoice>,
        /// Mapping from (dispute_id, arbitrator) to vote weight (stake)
        vote_weights: Mapping<(u128, AccountId), Balance>,
        /// Minimum stake to become arbitrator
        min_arbitrator_stake: Balance,
        /// Voting period duration (in milliseconds)
        voting_period: u64,
        /// Quorum percentage (in basis points)
        quorum_bps: u16,
        /// DAO owner/admin
        owner: AccountId,
    }

    /// Events
    #[ink(event)]
    pub struct DisputeCreated {
        #[ink(topic)]
        dispute_id: u128,
        #[ink(topic)]
        claimant: AccountId,
        #[ink(topic)]
        defendant: AccountId,
    }

    #[ink(event)]
    pub struct VoteCast {
        #[ink(topic)]
        dispute_id: u128,
        #[ink(topic)]
        arbitrator: AccountId,
        vote: VoteChoice,
        weight: Balance,
    }

    #[ink(event)]
    pub struct DisputeResolved {
        #[ink(topic)]
        dispute_id: u128,
        resolution: VoteChoice,
    }

    #[ink(event)]
    pub struct ArbitratorRegistered {
        #[ink(topic)]
        account: AccountId,
        stake: Balance,
    }

    #[ink(event)]
    pub struct DisputeAppealed {
        #[ink(topic)]
        dispute_id: u128,
        #[ink(topic)]
        appellant: AccountId,
    }

    /// Errors
    #[derive(Debug, PartialEq, Eq, Clone, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        DisputeNotFound,
        NotAuthorized,
        InsufficientStake,
        InvalidDisputeStatus,
        VotingPeriodEnded,
        VotingPeriodNotEnded,
        AlreadyVoted,
        NotRegisteredArbitrator,
        QuorumNotReached,
        DisputeCannotBeAppealed,
        TransferFailed,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl ArbitrationDAO {
        /// Constructor
        #[ink(constructor)]
        pub fn new(
            owner: AccountId,
            min_arbitrator_stake: Balance,
            voting_period_ms: u64,
            quorum_bps: u16,
        ) -> Self {
            Self {
                next_dispute_id: 0,
                disputes: Mapping::default(),
                arbitrators: Mapping::default(),
                votes: Mapping::default(),
                vote_weights: Mapping::default(),
                min_arbitrator_stake,
                voting_period: voting_period_ms,
                quorum_bps,
                owner,
            }
        }

        /// Register as an arbitrator
        #[ink(message, payable)]
        pub fn register_arbitrator(&mut self) -> Result<()> {
            let caller = self.env().caller();
            let stake = self.env().transferred_value();

            if stake < self.min_arbitrator_stake {
                return Err(Error::InsufficientStake);
            }

            let arbitrator = Arbitrator {
                account: caller,
                stake,
                disputes_participated: 0,
                disputes_resolved: 0,
                reputation: 100,
                is_active: true,
            };

            self.arbitrators.insert(caller, &arbitrator);

            self.env().emit_event(ArbitratorRegistered {
                account: caller,
                stake,
            });

            Ok(())
        }

        /// Increase arbitrator stake
        #[ink(message, payable)]
        pub fn increase_arbitrator_stake(&mut self) -> Result<()> {
            let caller = self.env().caller();
            let additional_stake = self.env().transferred_value();

            let mut arbitrator = self.arbitrators.get(caller)
                .ok_or(Error::NotRegisteredArbitrator)?;

            arbitrator.stake += additional_stake;
            self.arbitrators.insert(caller, &arbitrator);

            Ok(())
        }

        /// Create a new dispute
        #[ink(message)]
        pub fn create_dispute(
            &mut self,
            defendant: AccountId,
            description: ink::prelude::string::String,
            evidence_uri: ink::prelude::string::String,
        ) -> Result<u128> {
            let caller = self.env().caller();
            let dispute_id = self.next_dispute_id;
            self.next_dispute_id += 1;

            let now = self.env().block_timestamp();
            let voting_ends_at = now + self.voting_period;

            let dispute = Dispute {
                dispute_id,
                claimant: caller,
                defendant,
                description,
                evidence_uri,
                status: DisputeStatus::Open,
                created_at: now,
                voting_ends_at,
                votes_for_claimant: 0,
                votes_for_defendant: 0,
                resolution: None,
                can_appeal: true,
            };

            self.disputes.insert(dispute_id, &dispute);

            self.env().emit_event(DisputeCreated {
                dispute_id,
                claimant: caller,
                defendant,
            });

            Ok(dispute_id)
        }

        /// Start voting period
        #[ink(message)]
        pub fn start_voting(&mut self, dispute_id: u128) -> Result<()> {
            let caller = self.env().caller();
            let mut dispute = self.disputes.get(dispute_id)
                .ok_or(Error::DisputeNotFound)?;

            // Only claimant or defendant can start voting
            if caller != dispute.claimant && caller != dispute.defendant {
                return Err(Error::NotAuthorized);
            }

            if dispute.status != DisputeStatus::Open {
                return Err(Error::InvalidDisputeStatus);
            }

            dispute.status = DisputeStatus::Voting;
            self.disputes.insert(dispute_id, &dispute);

            Ok(())
        }

        /// Cast a vote
        #[ink(message)]
        pub fn vote(&mut self, dispute_id: u128, choice: VoteChoice) -> Result<()> {
            let caller = self.env().caller();

            // Check arbitrator is registered and active
            let mut arbitrator = self.arbitrators.get(caller)
                .ok_or(Error::NotRegisteredArbitrator)?;

            if !arbitrator.is_active {
                return Err(Error::NotRegisteredArbitrator);
            }

            let mut dispute = self.disputes.get(dispute_id)
                .ok_or(Error::DisputeNotFound)?;

            if dispute.status != DisputeStatus::Voting {
                return Err(Error::InvalidDisputeStatus);
            }

            // Check voting period
            if self.env().block_timestamp() > dispute.voting_ends_at {
                return Err(Error::VotingPeriodEnded);
            }

            // Check if already voted
            if self.votes.contains((dispute_id, caller)) {
                return Err(Error::AlreadyVoted);
            }

            // Record vote
            let vote_weight = arbitrator.stake;
            self.votes.insert((dispute_id, caller), &choice);
            self.vote_weights.insert((dispute_id, caller), &vote_weight);

            // Update vote counts
            match choice {
                VoteChoice::InFavorOfClaimant => dispute.votes_for_claimant += vote_weight,
                VoteChoice::InFavorOfDefendant => dispute.votes_for_defendant += vote_weight,
            }

            self.disputes.insert(dispute_id, &dispute);

            // Update arbitrator stats
            arbitrator.disputes_participated += 1;
            self.arbitrators.insert(caller, &arbitrator);

            self.env().emit_event(VoteCast {
                dispute_id,
                arbitrator: caller,
                vote: choice,
                weight: vote_weight,
            });

            Ok(())
        }

        /// Finalize dispute after voting period
        #[ink(message)]
        pub fn finalize_dispute(&mut self, dispute_id: u128) -> Result<VoteChoice> {
            let mut dispute = self.disputes.get(dispute_id)
                .ok_or(Error::DisputeNotFound)?;

            if dispute.status != DisputeStatus::Voting {
                return Err(Error::InvalidDisputeStatus);
            }

            // Check voting period ended
            if self.env().block_timestamp() <= dispute.voting_ends_at {
                return Err(Error::VotingPeriodNotEnded);
            }

            // Calculate total votes
            let total_votes = dispute.votes_for_claimant + dispute.votes_for_defendant;

            // Check quorum (simplified: at least one vote)
            if total_votes == 0 {
                return Err(Error::QuorumNotReached);
            }

            // Determine winner
            let resolution = if dispute.votes_for_claimant > dispute.votes_for_defendant {
                VoteChoice::InFavorOfClaimant
            } else {
                VoteChoice::InFavorOfDefendant
            };

            dispute.status = DisputeStatus::Resolved;
            dispute.resolution = Some(resolution.clone());
            self.disputes.insert(dispute_id, &dispute);

            self.env().emit_event(DisputeResolved {
                dispute_id,
                resolution: resolution.clone(),
            });

            Ok(resolution)
        }

        /// Appeal a dispute decision
        #[ink(message, payable)]
        pub fn appeal_dispute(&mut self, dispute_id: u128) -> Result<()> {
            let caller = self.env().caller();
            let mut dispute = self.disputes.get(dispute_id)
                .ok_or(Error::DisputeNotFound)?;

            // Only claimant or defendant can appeal
            if caller != dispute.claimant && caller != dispute.defendant {
                return Err(Error::NotAuthorized);
            }

            if dispute.status != DisputeStatus::Resolved {
                return Err(Error::InvalidDisputeStatus);
            }

            if !dispute.can_appeal {
                return Err(Error::DisputeCannotBeAppealed);
            }

            // Reset for new voting round
            dispute.status = DisputeStatus::Appealed;
            dispute.voting_ends_at = self.env().block_timestamp() + self.voting_period;
            dispute.votes_for_claimant = 0;
            dispute.votes_for_defendant = 0;
            dispute.can_appeal = false; // Only one appeal allowed

            self.disputes.insert(dispute_id, &dispute);

            self.env().emit_event(DisputeAppealed {
                dispute_id,
                appellant: caller,
            });

            Ok(())
        }

        /// Get dispute information
        #[ink(message)]
        pub fn get_dispute(&self, dispute_id: u128) -> Option<Dispute> {
            self.disputes.get(dispute_id)
        }

        /// Get arbitrator information
        #[ink(message)]
        pub fn get_arbitrator(&self, account: AccountId) -> Option<Arbitrator> {
            self.arbitrators.get(account)
        }

        /// Get vote for a dispute
        #[ink(message)]
        pub fn get_vote(&self, dispute_id: u128, arbitrator: AccountId) -> Option<VoteChoice> {
            self.votes.get((dispute_id, arbitrator))
        }

        /// Check if account is active arbitrator
        #[ink(message)]
        pub fn is_active_arbitrator(&self, account: AccountId) -> bool {
            self.arbitrators.get(account)
                .map(|a| a.is_active)
                .unwrap_or(false)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn register_arbitrator_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut contract = ArbitrationDAO::new(
                accounts.alice,
                100_000_000_000_000_000_000, // 100 GLIN
                7 * 24 * 60 * 60 * 1000,      // 7 days
                5000,                         // 50% quorum
            );

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
            ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(100_000_000_000_000_000_000);

            let result = contract.register_arbitrator();
            assert!(result.is_ok());

            let arbitrator = contract.get_arbitrator(accounts.bob).unwrap();
            assert_eq!(arbitrator.stake, 100_000_000_000_000_000_000);
            assert!(arbitrator.is_active);
        }

        #[ink::test]
        fn create_dispute_works() {
            let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
            let mut contract = ArbitrationDAO::new(
                accounts.alice,
                100_000_000_000_000_000_000,
                7 * 24 * 60 * 60 * 1000,
                5000,
            );

            ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);

            let result = contract.create_dispute(
                accounts.charlie,
                "Contract dispute".into(),
                "ipfs://evidence".into(),
            );

            assert!(result.is_ok());
            let dispute_id = result.unwrap();
            assert_eq!(dispute_id, 0);

            let dispute = contract.get_dispute(dispute_id).unwrap();
            assert_eq!(dispute.claimant, accounts.bob);
            assert_eq!(dispute.defendant, accounts.charlie);
            assert_eq!(dispute.status, DisputeStatus::Open);
        }
    }
}
