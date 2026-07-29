// IMPORTANT: Cross-contract wiring required after deployment
//
// `approve_milestone` calls `advance_level` on the progress contract to update
// a player's progress level atomically. This link is NOT automatic — after
// deploying both contracts you MUST run:
//
//   stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
//     -- set_progress_contract \
//     --progress_contract $PROGRESS_CONTRACT_ID
//
// The easiest way is to run `./scripts/initialize.sh` which does this for you.
// Without this step, milestones are recorded but player levels will NOT advance.

#![no_std]

mod errors;
mod events;
mod types;

use errors::VerificationError;
use types::{DataKey, DisputeVote, JuryConfig, Milestone, MilestoneDispute, Validator};

use soroban_sdk::{contract, contractimpl, Address, Env, String};

const DEFAULT_JURY_IMPACT_THRESHOLD: u64 = 100;
const DEFAULT_JURY_QUORUM: u32 = 3;
const DEFAULT_JURY_VOTING_WINDOW_SECS: u64 = 7 * 24 * 60 * 60;

// Generated client for the progress contract — used for cross-contract calls.
// The progress contract must be deployed and its address registered via
// `set_progress_contract` before `approve_milestone` can advance levels.
mod progress_contract {
    use scoutchain_shared_types::ProgressLevel;

    soroban_sdk::contractimport!(
        file = "../../target/wasm32v1-none/release/scoutchain_progress.wasm"
    );
}

#[contract]
pub struct VerificationContract;

#[contractimpl]
impl VerificationContract {
    // -------------------------------------------------------------------------
    // Admin
    // -------------------------------------------------------------------------

    pub fn initialize(env: Env, admin: Address) -> Result<(), VerificationError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(VerificationError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::JuryConfig, &Self::default_jury_config());
        Ok(())
    }

    /// Store the progress contract address so approve_milestone can call it.
    /// Must be called after both contracts are deployed (admin only).
    pub fn set_progress_contract(
        env: Env,
        progress_contract: Address,
    ) -> Result<(), VerificationError> {
        Self::require_admin(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::ProgressContract, &progress_contract);
        Ok(())
    }

    /// Configure the threshold and voting rules used by future disputes.
    pub fn set_jury_config(
        env: Env,
        impact_threshold: u64,
        quorum: u32,
        voting_window_secs: u64,
    ) -> Result<(), VerificationError> {
        Self::require_admin(&env)?;
        if quorum == 0 || voting_window_secs == 0 {
            return Err(VerificationError::InvalidJuryConfig);
        }

        env.storage().instance().set(
            &DataKey::JuryConfig,
            &JuryConfig {
                impact_threshold,
                quorum,
                voting_window_secs,
            },
        );
        Ok(())
    }

    /// Register a trusted validator (admin only).
    pub fn register_validator(
        env: Env,
        wallet: Address,
        credentials: String,
    ) -> Result<(), VerificationError> {
        Self::require_admin(&env)?;
        Self::require_not_paused(&env)?;

        if env
            .storage()
            .persistent()
            .has(&DataKey::Validator(wallet.clone()))
        {
            return Err(VerificationError::ValidatorAlreadyRegistered);
        }

        let validator = Validator {
            wallet: wallet.clone(),
            credentials,
            registered_at: env.ledger().timestamp(),
            active: true,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Validator(wallet.clone()), &validator);

        events::validator_registered(&env, &wallet);
        Ok(())
    }

    /// Deactivate a validator (admin only).
    pub fn revoke_validator(env: Env, wallet: Address) -> Result<(), VerificationError> {
        Self::require_admin(&env)?;
        let mut validator: Validator = env
            .storage()
            .persistent()
            .get(&DataKey::Validator(wallet.clone()))
            .ok_or(VerificationError::ValidatorNotFound)?;
        validator.active = false;
        env.storage()
            .persistent()
            .set(&DataKey::Validator(wallet.clone()), &validator);
        events::validator_revoked(&env, &wallet);
        Ok(())
    }

    pub fn pause_contract(env: Env) -> Result<(), VerificationError> {
        Self::require_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        Ok(())
    }

    pub fn unpause_contract(env: Env) -> Result<(), VerificationError> {
        Self::require_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Milestone approval
    // -------------------------------------------------------------------------

    /// Approve a player milestone. Caller must be a registered, active validator.
    ///
    /// After storing the milestone, this function calls `progress.advance_level`
    /// on the registered progress contract so both state changes happen atomically
    /// in the same Stellar transaction.
    ///
    /// Each milestone records the Stellar ledger sequence number for
    /// tamper-proof auditability.
    ///
    /// Returns the milestone index for this player.
    pub fn approve_milestone(
        env: Env,
        validator_wallet: Address,
        player_id: u64,
        description: String,
        evidence_hash: String,
    ) -> Result<u32, VerificationError> {
        Self::require_not_paused(&env)?;
        validator_wallet.require_auth();

        // Verify the caller is an active validator
        let validator: Validator = env
            .storage()
            .persistent()
            .get(&DataKey::Validator(validator_wallet.clone()))
            .ok_or(VerificationError::ValidatorNotFound)?;

        if !validator.active {
            return Err(VerificationError::ValidatorInactive);
        }

        // Increment milestone counter for this player
        let counter_key = DataKey::MilestoneCounter(player_id);
        let index: u32 = env.storage().persistent().get(&counter_key).unwrap_or(0u32);
        let next_index = index.checked_add(1).ok_or(VerificationError::Overflow)?;

        let milestone = Milestone {
            player_id,
            validator: validator_wallet.clone(),
            description,
            evidence_hash,
            approved_at: env.ledger().timestamp(),
            ledger_sequence: env.ledger().sequence(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Milestone(player_id, next_index), &milestone);
        env.storage().persistent().set(&counter_key, &next_index);

        // Increment per-validator milestone count
        let val_key = DataKey::ValidatorMilestoneCount(validator_wallet.clone());
        let val_count: u32 = env.storage().persistent().get(&val_key).unwrap_or(0u32);
        env.storage().persistent().set(
            &val_key,
            &val_count
                .checked_add(1)
                .ok_or(VerificationError::Overflow)?,
        );

        events::milestone_approved(
            &env,
            player_id,
            &validator_wallet,
            next_index,
            &milestone.description,
            &milestone.evidence_hash,
        );

        // Cross-contract call: advance the player's progress level.
        // This is a best-effort call — if the progress contract is not set
        // (e.g. during testing without a full deployment), we skip it.
        // In production, always call set_progress_contract before going live.
        if let Some(progress_addr) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::ProgressContract)
        {
            let progress_client = progress_contract::Client::new(&env, &progress_addr);
            // advance_level will return AlreadyAtMaxLevel if the player is
            // already at EliteTier — we intentionally ignore that error here
            // so the milestone is still recorded even at max level.
            let _ = progress_client.try_advance_level(&validator_wallet, &player_id, &next_index);
        }

        Ok(next_index)
    }

    // -------------------------------------------------------------------------
    // Milestone disputes
    // -------------------------------------------------------------------------

    /// File a dispute against an approved milestone.
    ///
    /// High-impact disputes are escalated to the validator jury. Lower-impact
    /// disputes retain the existing admin-resolution path.
    pub fn dispute_milestone(
        env: Env,
        filed_by: Address,
        player_id: u64,
        milestone_index: u32,
        reason: String,
        impact_score: u64,
    ) -> Result<(), VerificationError> {
        Self::require_not_paused(&env)?;
        filed_by.require_auth();

        if !env
            .storage()
            .persistent()
            .has(&DataKey::Milestone(player_id, milestone_index))
        {
            return Err(VerificationError::InvalidInput);
        }

        let dispute_key = DataKey::MilestoneDispute(player_id, milestone_index);
        if env.storage().persistent().has(&dispute_key) {
            return Err(VerificationError::DisputeAlreadyExists);
        }

        let config = Self::jury_config(&env);
        let jury_required = impact_score >= config.impact_threshold;
        let filed_at = env.ledger().timestamp();
        let voting_deadline = if jury_required {
            filed_at
                .checked_add(config.voting_window_secs)
                .ok_or(VerificationError::Overflow)?
        } else {
            filed_at
        };
        let dispute = MilestoneDispute {
            player_id,
            milestone_index,
            filed_by: filed_by.clone(),
            reason,
            impact_score,
            filed_at,
            voting_deadline,
            jury_required,
            quorum: config.quorum,
            resolved: false,
            upheld: false,
            votes_for: 0,
            votes_against: 0,
        };

        env.storage().persistent().set(&dispute_key, &dispute);
        events::milestone_disputed(&env, player_id, milestone_index, &filed_by, jury_required);
        Ok(())
    }

    /// Resolve a low-impact dispute through the backwards-compatible admin path.
    pub fn resolve_dispute(
        env: Env,
        player_id: u64,
        milestone_index: u32,
        upheld: bool,
    ) -> Result<(), VerificationError> {
        Self::require_admin(&env)?;
        let dispute_key = DataKey::MilestoneDispute(player_id, milestone_index);
        let mut dispute: MilestoneDispute = env
            .storage()
            .persistent()
            .get(&dispute_key)
            .ok_or(VerificationError::DisputeNotFound)?;
        if dispute.resolved {
            return Err(VerificationError::DisputeAlreadyResolved);
        }
        if dispute.jury_required {
            return Err(VerificationError::DisputeRequiresJury);
        }

        dispute.resolved = true;
        dispute.upheld = upheld;
        env.storage().persistent().set(&dispute_key, &dispute);
        events::dispute_resolved(&env, player_id, milestone_index, upheld);
        Ok(())
    }

    /// Cast one immutable vote on a high-impact dispute.
    pub fn cast_dispute_vote(
        env: Env,
        validator_wallet: Address,
        player_id: u64,
        milestone_index: u32,
        upheld: bool,
    ) -> Result<(), VerificationError> {
        Self::require_not_paused(&env)?;
        validator_wallet.require_auth();
        Self::require_active_validator(&env, &validator_wallet)?;

        let dispute_key = DataKey::MilestoneDispute(player_id, milestone_index);
        let mut dispute: MilestoneDispute = env
            .storage()
            .persistent()
            .get(&dispute_key)
            .ok_or(VerificationError::DisputeNotFound)?;
        if dispute.resolved {
            return Err(VerificationError::DisputeAlreadyResolved);
        }
        if !dispute.jury_required {
            return Err(VerificationError::DisputeDoesNotRequireJury);
        }
        if env.ledger().timestamp() >= dispute.voting_deadline {
            return Err(VerificationError::VotingWindowClosed);
        }

        let milestone: Milestone = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(player_id, milestone_index))
            .ok_or(VerificationError::InvalidInput)?;
        if milestone.validator == validator_wallet {
            return Err(VerificationError::ConflictedValidator);
        }

        let vote_key = DataKey::DisputeVote(player_id, milestone_index, validator_wallet.clone());
        if env.storage().persistent().has(&vote_key) {
            return Err(VerificationError::VoteAlreadyCast);
        }

        if upheld {
            dispute.votes_for = dispute
                .votes_for
                .checked_add(1)
                .ok_or(VerificationError::Overflow)?;
        } else {
            dispute.votes_against = dispute
                .votes_against
                .checked_add(1)
                .ok_or(VerificationError::Overflow)?;
        }
        let vote = DisputeVote {
            validator: validator_wallet.clone(),
            upheld,
            cast_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&vote_key, &vote);
        env.storage().persistent().set(&dispute_key, &dispute);
        events::dispute_vote_cast(&env, player_id, milestone_index, &validator_wallet, upheld);
        Ok(())
    }

    /// Finalize a jury dispute once it has a decisive quorum or its window ends.
    /// Anyone may call this function, so no administrator can suppress a result.
    pub fn tally_dispute(
        env: Env,
        player_id: u64,
        milestone_index: u32,
    ) -> Result<bool, VerificationError> {
        let dispute_key = DataKey::MilestoneDispute(player_id, milestone_index);
        let mut dispute: MilestoneDispute = env
            .storage()
            .persistent()
            .get(&dispute_key)
            .ok_or(VerificationError::DisputeNotFound)?;
        if dispute.resolved {
            return Err(VerificationError::DisputeAlreadyResolved);
        }
        if !dispute.jury_required {
            return Err(VerificationError::DisputeDoesNotRequireJury);
        }

        let total_votes = dispute
            .votes_for
            .checked_add(dispute.votes_against)
            .ok_or(VerificationError::Overflow)?;
        let deadline_passed = env.ledger().timestamp() >= dispute.voting_deadline;
        let decisive_quorum =
            total_votes >= dispute.quorum && dispute.votes_for != dispute.votes_against;
        if !decisive_quorum && !deadline_passed {
            return Err(VerificationError::TallyNotReady);
        }

        // A tie, or a deadline with no quorum, preserves the original milestone.
        let upheld = total_votes >= dispute.quorum && dispute.votes_for > dispute.votes_against;
        dispute.resolved = true;
        dispute.upheld = upheld;
        env.storage().persistent().set(&dispute_key, &dispute);
        events::dispute_tallied(
            &env,
            player_id,
            milestone_index,
            upheld,
            dispute.votes_for,
            dispute.votes_against,
        );
        Ok(upheld)
    }

    // -------------------------------------------------------------------------
    // Queries
    // -------------------------------------------------------------------------

    pub fn get_milestone(
        env: Env,
        player_id: u64,
        index: u32,
    ) -> Result<Milestone, VerificationError> {
        env.storage()
            .persistent()
            .get(&DataKey::Milestone(player_id, index))
            .ok_or(VerificationError::InvalidInput)
    }

    pub fn get_milestone_count(env: Env, player_id: u64) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::MilestoneCounter(player_id))
            .unwrap_or(0u32)
    }

    pub fn get_jury_config(env: Env) -> JuryConfig {
        Self::jury_config(&env)
    }

    pub fn get_dispute(
        env: Env,
        player_id: u64,
        milestone_index: u32,
    ) -> Result<MilestoneDispute, VerificationError> {
        env.storage()
            .persistent()
            .get(&DataKey::MilestoneDispute(player_id, milestone_index))
            .ok_or(VerificationError::DisputeNotFound)
    }

    pub fn get_dispute_vote(
        env: Env,
        player_id: u64,
        milestone_index: u32,
        validator_wallet: Address,
    ) -> Result<DisputeVote, VerificationError> {
        env.storage()
            .persistent()
            .get(&DataKey::DisputeVote(
                player_id,
                milestone_index,
                validator_wallet,
            ))
            .ok_or(VerificationError::InvalidInput)
    }

    pub fn get_dispute_votes(
        env: Env,
        player_id: u64,
        milestone_index: u32,
    ) -> Result<(u32, u32), VerificationError> {
        let dispute = Self::get_dispute(env, player_id, milestone_index)?;
        Ok((dispute.votes_for, dispute.votes_against))
    }

    pub fn get_validator_milestone_count(env: Env, wallet: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::ValidatorMilestoneCount(wallet))
            .unwrap_or(0u32)
    }

    pub fn get_validator(env: Env, wallet: Address) -> Result<Validator, VerificationError> {
        env.storage()
            .persistent()
            .get(&DataKey::Validator(wallet))
            .ok_or(VerificationError::ValidatorNotFound)
    }

    pub fn is_active_validator(env: Env, wallet: Address) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, Validator>(&DataKey::Validator(wallet))
            .map(|v| v.active)
            .unwrap_or(false)
    }

    pub fn health(env: Env) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Initialized)
            .unwrap_or(false)
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn require_admin(env: &Env) -> Result<(), VerificationError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VerificationError::NotInitialized)?;
        admin.require_auth();
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), VerificationError> {
        if env
            .storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Paused)
            .unwrap_or(false)
        {
            return Err(VerificationError::ContractPaused);
        }
        Ok(())
    }

    fn require_active_validator(env: &Env, wallet: &Address) -> Result<(), VerificationError> {
        let validator: Validator = env
            .storage()
            .persistent()
            .get(&DataKey::Validator(wallet.clone()))
            .ok_or(VerificationError::ValidatorNotFound)?;
        if !validator.active {
            return Err(VerificationError::ValidatorInactive);
        }
        Ok(())
    }

    fn jury_config(env: &Env) -> JuryConfig {
        env.storage()
            .instance()
            .get(&DataKey::JuryConfig)
            .unwrap_or_else(Self::default_jury_config)
    }

    fn default_jury_config() -> JuryConfig {
        JuryConfig {
            impact_threshold: DEFAULT_JURY_IMPACT_THRESHOLD,
            quorum: DEFAULT_JURY_QUORUM,
            voting_window_secs: DEFAULT_JURY_VOTING_WINDOW_SECS,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, EnvTestConfig, Ledger as _},
        Env, String,
    };

    fn setup() -> (Env, VerificationContractClient<'static>) {
        let env = Env::new_with_config(EnvTestConfig {
            capture_snapshot_at_drop: false,
        });
        env.mock_all_auths();
        let id = env.register_contract(None, VerificationContract);
        let client = VerificationContractClient::new(&env, &id);
        (env, client)
    }

    #[test]
    fn test_validator_milestone_count() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let validator = Address::generate(&env);
        client.register_validator(&validator, &String::from_str(&env, "Coach"));

        // Unknown wallet returns 0
        assert_eq!(
            client.get_validator_milestone_count(&Address::generate(&env)),
            0
        );

        for i in 1u64..=3 {
            client.approve_milestone(
                &validator,
                &i,
                &String::from_str(&env, "milestone"),
                &String::from_str(&env, "QmEvidence"),
            );
        }

        assert_eq!(client.get_validator_milestone_count(&validator), 3);
    }

    #[test]
    fn test_health_false_before_initialize() {
        let (_env, client) = setup();
        assert!(!client.health());
    }

    #[test]
    fn test_register_and_approve() {
        let (env, client) = setup();
        env.ledger().with_mut(|ledger| ledger.sequence_number = 1);
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let validator = Address::generate(&env);
        client.register_validator(&validator, &String::from_str(&env, "UEFA B License"));

        assert!(client.is_active_validator(&validator));

        // No progress contract set — approve_milestone still records the milestone
        let idx = client.approve_milestone(
            &validator,
            &1u64,
            &String::from_str(&env, "Scored 5 goals in Local Cup"),
            &String::from_str(&env, "QmEvidence123"),
        );
        assert_eq!(idx, 1);
        assert_eq!(client.get_milestone_count(&1u64), 1);

        let milestone = client.get_milestone(&1u64, &1);
        assert!(milestone.ledger_sequence > 0);
    }

    fn setup_jury_dispute(
        impact_score: u64,
        quorum: u32,
        voting_window_secs: u64,
    ) -> (
        Env,
        VerificationContractClient<'static>,
        Address,
        Address,
        Address,
        Address,
    ) {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);
        client.set_jury_config(&100, &quorum, &voting_window_secs);

        let original_approver = Address::generate(&env);
        let voter_one = Address::generate(&env);
        let voter_two = Address::generate(&env);
        let voter_three = Address::generate(&env);
        for validator in [&original_approver, &voter_one, &voter_two, &voter_three] {
            client.register_validator(validator, &String::from_str(&env, "Coach"));
        }

        client.approve_milestone(
            &original_approver,
            &1,
            &String::from_str(&env, "Regional tournament result"),
            &String::from_str(&env, "QmEvidence"),
        );
        let filer = Address::generate(&env);
        client.dispute_milestone(
            &filer,
            &1,
            &1,
            &String::from_str(&env, "Evidence is disputed"),
            &impact_score,
        );

        (
            env,
            client,
            original_approver,
            voter_one,
            voter_two,
            voter_three,
        )
    }

    #[test]
    fn test_jury_quorum_upholds_dispute() {
        let (env, client, _, voter_one, voter_two, voter_three) = setup_jury_dispute(100, 3, 100);

        for voter in [&voter_one, &voter_two, &voter_three] {
            client.cast_dispute_vote(voter, &1, &1, &true);
        }

        assert_eq!(client.get_dispute_votes(&1, &1), (3, 0));
        assert!(client.tally_dispute(&1, &1));
        let dispute = client.get_dispute(&1, &1);
        assert!(dispute.resolved);
        assert!(dispute.upheld);
        assert_eq!(dispute.voting_deadline, env.ledger().timestamp() + 100);
    }

    #[test]
    fn test_jury_quorum_rejects_dispute() {
        let (_, client, _, voter_one, voter_two, voter_three) = setup_jury_dispute(100, 3, 100);

        client.cast_dispute_vote(&voter_one, &1, &1, &false);
        client.cast_dispute_vote(&voter_two, &1, &1, &false);
        client.cast_dispute_vote(&voter_three, &1, &1, &true);

        assert!(!client.tally_dispute(&1, &1));
        let dispute = client.get_dispute(&1, &1);
        assert!(dispute.resolved);
        assert!(!dispute.upheld);
        assert_eq!(client.get_dispute_votes(&1, &1), (1, 2));
    }

    #[test]
    fn test_jury_tie_rejects_after_voting_window() {
        let (env, client, _, voter_one, voter_two, _) = setup_jury_dispute(100, 2, 100);

        client.cast_dispute_vote(&voter_one, &1, &1, &true);
        client.cast_dispute_vote(&voter_two, &1, &1, &false);
        assert!(client.try_tally_dispute(&1, &1).is_err());

        env.ledger().with_mut(|ledger| ledger.timestamp += 100);
        assert!(!client.tally_dispute(&1, &1));
        assert!(!client.get_dispute(&1, &1).upheld);
    }

    #[test]
    fn test_conflicted_validator_cannot_vote_on_own_milestone() {
        let (_, client, original_approver, _, _, _) = setup_jury_dispute(100, 3, 100);

        assert!(client
            .try_cast_dispute_vote(&original_approver, &1, &1, &true)
            .is_err());
        assert_eq!(client.get_dispute_votes(&1, &1), (0, 0));
    }

    #[test]
    fn test_low_impact_dispute_retains_admin_resolution() {
        let (_, client, _, _, _, _) = setup_jury_dispute(99, 3, 100);

        let dispute = client.get_dispute(&1, &1);
        assert!(!dispute.jury_required);
        client.resolve_dispute(&1, &1, &true);
        let resolved = client.get_dispute(&1, &1);
        assert!(resolved.resolved);
        assert!(resolved.upheld);
    }

    #[test]
    fn test_multiple_milestones_same_player() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let validator = Address::generate(&env);
        client.register_validator(&validator, &String::from_str(&env, "Coach"));

        let idx1 = client.approve_milestone(
            &validator,
            &1u64,
            &String::from_str(&env, "Identity verified"),
            &String::from_str(&env, "QmKYC"),
        );
        let idx2 = client.approve_milestone(
            &validator,
            &1u64,
            &String::from_str(&env, "Top speed 32 km/h"),
            &String::from_str(&env, "QmSpeed"),
        );
        assert_eq!(idx1, 1);
        assert_eq!(idx2, 2);
        assert_eq!(client.get_milestone_count(&1u64), 2);
    }

    #[test]
    fn test_revoke_validator() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let validator = Address::generate(&env);
        client.register_validator(&validator, &String::from_str(&env, "Coach"));
        client.revoke_validator(&validator);

        assert!(!client.is_active_validator(&validator));
    }

    #[test]
    #[should_panic]
    fn test_revoked_validator_cannot_approve() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let validator = Address::generate(&env);
        client.register_validator(&validator, &String::from_str(&env, "Coach"));
        client.revoke_validator(&validator);

        // Should panic — validator is inactive
        client.approve_milestone(
            &validator,
            &1u64,
            &String::from_str(&env, "Some milestone"),
            &String::from_str(&env, "QmEvidence"),
        );
    }

    #[test]
    #[should_panic]
    fn test_unregistered_validator_cannot_approve() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let random = Address::generate(&env);
        // Should panic — not in validator registry
        client.approve_milestone(
            &random,
            &1u64,
            &String::from_str(&env, "Some milestone"),
            &String::from_str(&env, "QmEvidence"),
        );
    }

    #[test]
    fn test_two_validators_approve_milestones_for_same_player() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let validator1 = Address::generate(&env);
        let validator2 = Address::generate(&env);
        client.register_validator(&validator1, &String::from_str(&env, "Coach A"));
        client.register_validator(&validator2, &String::from_str(&env, "Coach B"));

        client.approve_milestone(
            &validator1,
            &1u64,
            &String::from_str(&env, "Identity verified"),
            &String::from_str(&env, "QmEvidence1"),
        );
        client.approve_milestone(
            &validator2,
            &1u64,
            &String::from_str(&env, "Top speed 32 km/h"),
            &String::from_str(&env, "QmEvidence2"),
        );

        assert_eq!(client.get_milestone_count(&1u64), 2);

        let m1 = client.get_milestone(&1u64, &1);
        let m2 = client.get_milestone(&1u64, &2);
        assert_eq!(m1.validator, validator1);
        assert_eq!(m2.validator, validator2);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn test_approve_milestone_blocked_when_paused() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let validator = Address::generate(&env);
        client.register_validator(&validator, &String::from_str(&env, "Coach"));

        client.pause_contract();

        // Should panic — contract is paused
        client.approve_milestone(
            &validator,
            &1u64,
            &String::from_str(&env, "Some milestone"),
            &String::from_str(&env, "QmEvidence"),
        );
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #13)")]
    fn test_approve_milestone_overflow() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let validator = Address::generate(&env);
        client.register_validator(&validator, &String::from_str(&env, "Coach"));

        // Pre-set the counter to u32::MAX so the next increment overflows
        env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .set(&DataKey::MilestoneCounter(1u64), &u32::MAX);
        });

        // Should return Overflow (#13) instead of panicking with expect()
        client.approve_milestone(
            &validator,
            &1u64,
            &String::from_str(&env, "overflow test"),
            &String::from_str(&env, "QmHash"),
        );
    }

    #[test]
    #[should_panic]
    fn test_get_validator_not_found() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let unknown = Address::generate(&env);
        client.get_validator(&unknown);
    }
}
