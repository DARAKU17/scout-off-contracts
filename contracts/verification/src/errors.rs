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
    DisputeAlreadyExists = 14,
    DisputeNotFound = 15,
    DisputeAlreadyResolved = 16,
    DisputeRequiresJury = 17,
    DisputeDoesNotRequireJury = 18,
    ConflictedValidator = 19,
    VoteAlreadyCast = 20,
    VotingWindowClosed = 21,
    TallyNotReady = 22,
    InvalidJuryConfig = 23,
}
