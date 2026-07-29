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
    /// Admin-verified organization the validator represents.
    pub affiliation: String,
    pub registered_at: u64,
    pub active: bool,
}

/// Rules that gate level-advancing milestones on independent organizations.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DiversityConfig {
    /// Minimum number of affiliations required to advance at or above the gate.
    pub min_distinct_affiliations: u32,
    /// First milestone index that requires organizational diversity.
    pub gated_milestone_index: u32,
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
    /// Diversity rules for level-advancing milestones.
    DiversityConfig,
    /// (player_id, affiliation) → whether that affiliation has approved a milestone.
    PlayerAffiliationUsed(u64, String),
    /// player_id → number of distinct affiliations that approved milestones.
    PlayerAffiliationCount(u64),
}
