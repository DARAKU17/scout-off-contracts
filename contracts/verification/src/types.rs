use soroban_sdk::{contracttype, Address, String};

/// A single verified milestone record
#[contracttype]
#[derive(Clone, Debug)]
pub struct Milestone {
    pub player_id: u64,
    pub validator: Address,
    pub description: String,
    /// IPFS/Arweave CID of supporting evidence (video clip, stat sheet, etc.)
    pub evidence_hash: String,
    pub approved_at: u64,
    /// Stellar ledger sequence at time of approval for tamper-proof auditability
    pub ledger_sequence: u32,
}

/// Validator entry in the trusted registry
#[contracttype]
#[derive(Clone, Debug)]
pub struct Validator {
    pub wallet: Address,
    /// Human-readable credential label (e.g. "UEFA B License", "Academy Director")
    pub credentials: String,
    pub registered_at: u64,
    pub active: bool,
}

/// Whether a validator revocation requires re-review of prior approvals.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevocationSeverity {
    Routine,
    ForCause,
}

/// The reason and severity recorded when a validator is revoked.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RevocationRecord {
    pub severity: RevocationSeverity,
    pub reason: String,
    pub revoked_at: u64,
}

/// Identifies one milestone in a validator's approval history.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidatorMilestoneRef {
    pub player_id: u64,
    pub milestone_index: u32,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Paused,
    /// validator wallet → Validator
    Validator(Address),
    /// milestone counter per player
    MilestoneCounter(u64),
    /// (player_id, milestone_index) → Milestone
    Milestone(u64, u32),
    /// registration contract address (cross-contract calls)
    RegistrationContract,
    /// progress contract address (cross-contract calls)
    ProgressContract,
    /// milestone count per validator wallet
    ValidatorMilestoneCount(Address),
    /// (validator wallet, approval index) → approved milestone reference
    ValidatorMilestone(Address, u32),
    /// validator wallet → RevocationRecord
    ValidatorRevocation(Address),
    /// (player_id, milestone_index) → whether re-review is pending
    MilestonePendingReReview(u64, u32),
}
