pub use scoutchain_shared_types::ContractHealth;
use soroban_sdk::{contracttype, Address, String};

/// Richer validator status — distinguishes unregistered from revoked.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ValidatorStatus {
    NotRegistered,
    Active,
    Revoked,
}

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

/// A dispute filed against an approved milestone.
/// Disputes are purely informational records — filing one does not
/// automatically reverse the approval or affect the progress level.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MilestoneDispute {
    /// The player whose milestone is under dispute
    pub player_id: u64,
    /// Index of the disputed milestone (1-based)
    pub milestone_index: u32,
    /// The validator who approved the disputed milestone
    pub validator: Address,
    /// Free-form reason provided by the disputing party
    pub reason: String,
    /// Address of the account that filed the dispute
    pub filed_by: Address,
    /// Ledger timestamp at the time of filing
    pub filed_at: u64,
}

/// Lightweight reference used by paginated dispute queries.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DisputeRef {
    pub player_id: u64,
    pub milestone_index: u32,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Paused,
    ProgressContract,
    ProgressContractSet,
    Validator(Address),
    MilestoneCounter(u64),
    Milestone(u64, u32),
    ValidatorMilestoneCount(Address),
    ValidatorVector,
    TotalMilestoneCount,
    /// Per-validator milestone reference list: validator → Vec<DisputeRef>
    /// Stores (player_id, milestone_index) pairs in approval order.
    ValidatorMilestones(Address),
    /// Per-milestone dispute record: (player_id, milestone_index) → MilestoneDispute
    DisputeRecord(u64, u32),
    /// Per-player-validator milestone count cap key
    ValidatorPlayerMilestoneCount(Address, u64),
    /// Global count of active (unresolved) disputes
    ActiveDisputeCount,
}
