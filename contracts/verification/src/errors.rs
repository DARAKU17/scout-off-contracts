use scoutchain_shared_types::AdminError;
use soroban_sdk::contracterror;

/// Append-only: do not renumber existing variants. See docs/CONTRIBUTING.md.
#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum VerificationError {
    // ── Initialization & lifecycle ──
    /// `initialize` called more than once.
    AlreadyInitialized = 1,
    /// Operation before `initialize`.
    NotInitialized = 2,
    /// Circuit breaker is active.
    ContractPaused = 3,
    /// `set_progress_contract` called twice.
    AlreadyConfigured = 11,

    // ── Authorization ──
    /// Wrong account for a privileged operation.
    Unauthorized = 4,

    // ── Validators ──
    /// Wallet not in validator registry.
    ValidatorNotFound = 5,
    /// Validator has been revoked.
    ValidatorInactive = 6,
    /// Wallet already registered as validator.
    ValidatorAlreadyRegistered = 7,
    /// 100-validator limit reached; contract upgrade required to raise the cap.
    ValidatorCapReached = 15,

    // ── Milestones & evidence ──
    /// Invalid `player_id`.
    PlayerNotFound = 8,
    /// Index out of range.
    MilestoneNotFound = 14,
    /// Evidence hash has already been used in a prior `approve_milestone` call.
    DuplicateEvidence = 16,
    /// Validator has already approved 5 milestones for this player.
    MilestoneLimitExceeded = 17,
    /// Dispute was already resolved and cannot be resolved again.
    DisputeAlreadyResolved = 18,

    // ── Input validation ──
    /// Bad evidence hash or credentials too long.
    InvalidInput = 9,
    /// Revocation reason exceeds 128 bytes.
    ReasonTooLong = 10,

    // ── Cross-contract & arithmetic ──
    /// Cross-contract `advance_level` failed.
    ProgressCallFailed = 12,
    /// Milestone counter overflowed.
    Overflow = 13,

    // ── Admin transfer ──
    /// `accept_admin` called before an admin transfer was proposed.
    PendingAdminNotSet = 19,

    // ── Function-scoped pausing ──
    /// The approve_milestone function is paused independently of whole-contract pause.
    ApproveMilestonePaused = 20,

    // ── Credential attestation ──
    /// The provided attestation signature is invalid or does not match the expected issuer.
    InvalidAttestation = 21,
    /// The attestation issuer is not registered in the trusted issuer registry.
    UntrustedIssuer = 22,
    /// The credential claim has expired.
    CredentialExpired = 23,
    /// The issuer registry limit has been reached; contract upgrade required to raise the cap.
    IssuerCapReached = 24,
    /// The issuer is already registered.
    IssuerAlreadyRegistered = 25,
    /// The issuer was not found in the registry.
    IssuerNotFound = 26,
    // ── Specialization ──
    /// Validator is not tagged for the requested milestone category.
    /// Only raised when a `milestone_category` is supplied to `approve_milestone`
    /// and the validator's `specializations` list does not contain that category.
    SpecializationMismatch = 21,
}

impl AdminError for VerificationError {
    fn not_initialized() -> Self {
        VerificationError::NotInitialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    const VALID_CID_V0: &str = "QmPK1s3pNYLi9ERiq3BDxKa4XosgWwFRQUydHUtz4YgpqB";

    fn setup() -> (Env, crate::VerificationContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, crate::VerificationContract);
        let client = crate::VerificationContractClient::new(&env, &id);
        (env, client)
    }

    #[test]
    fn test_approve_milestone_description_at_boundary_succeeds() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        let validator = Address::generate(&env);
        client.initialize(&admin);
        client.register_validator(&validator, &String::from_str(&env, "UEFA B License"), &Vec::new(&env));

        let description_256 = String::from_str(&env, &"a".repeat(256));
        let evidence = String::from_str(&env, VALID_CID_V0);

        let result = client.try_approve_milestone(&validator, &1u64, &description_256, &evidence);
        assert!(result.is_ok(), "256-byte description should succeed");
    }

    #[test]
    fn test_approve_milestone_description_over_limit_returns_invalid_input() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        let validator = Address::generate(&env);
        client.initialize(&admin);
        client.register_validator(&validator, &String::from_str(&env, "UEFA B License"), &Vec::new(&env));

        let description_257 = String::from_str(&env, &"a".repeat(257));
        let evidence = String::from_str(&env, VALID_CID_V0);

        let result = client.try_approve_milestone(&validator, &1u64, &description_257, &evidence);
        assert_eq!(
            result,
            Err(Ok(VerificationError::InvalidInput)),
            "257-byte description should return InvalidInput"
        );
    }
}
