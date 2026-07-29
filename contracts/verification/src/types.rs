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

/// Admin-configured rules for escalating high-impact disputes to a validator jury.
#[contracttype]
#[derive(Clone, Debug)]
pub struct JuryConfig {
    /// Disputes with an impact score at or above this value require a jury vote.
    pub impact_threshold: u64,
    /// Minimum number of votes required for an upheld jury outcome.
    pub quorum: u32,
    /// Number of seconds validators have to vote after a dispute is filed.
    pub voting_window_secs: u64,
}

/// An on-chain record of a disputed milestone and, when required, its jury tally.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MilestoneDispute {
    pub player_id: u64,
    pub milestone_index: u32,
    pub filed_by: Address,
    pub reason: String,
    pub impact_score: u64,
    pub filed_at: u64,
    pub voting_deadline: u64,
    pub jury_required: bool,
    /// Quorum fixed when the dispute is filed so later config changes cannot alter it.
    pub quorum: u32,
    pub resolved: bool,
    pub upheld: bool,
    pub votes_for: u32,
    pub votes_against: u32,
}

/// An individual validator's immutable vote on a disputed milestone.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DisputeVote {
    pub validator: Address,
    pub upheld: bool,
    pub cast_at: u64,
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
    /// Jury escalation threshold, quorum, and voting-window configuration.
    JuryConfig,
    /// (player_id, milestone_index) → MilestoneDispute
    MilestoneDispute(u64, u32),
    /// (player_id, milestone_index, validator) → DisputeVote
    DisputeVote(u64, u32, Address),
}
