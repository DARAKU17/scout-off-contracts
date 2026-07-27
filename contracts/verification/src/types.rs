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
    /// Geographic region of this validator (e.g. "West Africa", "South America").
    /// Used by the region-quorum check to ensure milestone diversity.
    /// Max 128 bytes, same limit as ScoutProfile.region.
    pub region: String,
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
    ValidatorPlayerMilestoneCount(Address, u64),
    ValidatorVector,
    TotalMilestoneCount,
    /// Minimum number of distinct validator regions required before approve_milestone
    /// may call advance_level for Level-2 (PerformanceMilestones) and Level-3
    /// (EliteTier) transitions. Default 0 means the check is disabled.
    MinRegionQuorum,
}
