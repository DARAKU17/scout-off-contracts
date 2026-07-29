pub use scoutchain_shared_types::ContractHealth;
use soroban_sdk::{contracttype, Address, String, Vec};

const MAX_ISSUERS: u32 = 20;

/// A trusted credential issuer authorized to sign validator attestation claims.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Issuer {
    /// Issuer wallet address (also the ed25519 public key holder).
    pub wallet: Address,
    /// Human-readable issuer name (e.g. "Football Federation", "UEFA").
    pub name: String,
    /// Ledger timestamp when the issuer was registered.
    pub registered_at: u64,
    /// Whether this issuer is currently authorized to sign attestations.
    pub active: bool,
}

/// A signed credential claim produced by an issuer off-chain.
#[contracttype]
#[derive(Clone, Debug)]
pub struct CredentialAttestation {
    /// Wallet address of the issuer who signed this attestation.
    pub issuer_wallet: Address,
    /// Validator wallet being attested.
    pub validator_wallet: Address,
    /// Credential type label (e.g. "UEFA B License").
    pub credential_type: String,
    /// Unix timestamp when the credential expires (0 = no expiry).
    pub expires_at: u64,
    /// ed25519 signature over (issuer_wallet || validator_wallet || credential_type || expires_at).
    pub signature: Vec<u8>,
/// Convenience aggregate returned by `get_validator_activity_report`.
///
/// Bundles the data from four individual queries into one call:
/// - `get_validator`               → credentials, registered_at, active
/// - `get_validator_status`        → status
/// - `get_validator_milestone_count` → milestone_count
/// - `get_validator_players`       → distinct_players (and distinct_player_count)
///
/// This is a pure read-only aggregate — no new storage or business logic.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidatorActivityReport {
    /// Validator wallet address.
    pub wallet: Address,
    /// Human-readable credential label set at registration time.
    pub credentials: String,
    /// Unix timestamp (seconds) when the validator was registered.
    pub registered_at: u64,
    /// Whether the validator is currently active.
    pub active: bool,
    /// Richer status distinguishing Active / Revoked / RevokedForCause / NotRegistered.
    pub status: ValidatorStatus,
    /// Total number of milestones approved by this validator across all players.
    pub milestone_count: u32,
    /// Number of distinct players for whom this validator has approved at least one milestone.
    pub distinct_player_count: u32,
    /// List of distinct player IDs (same data as `get_validator_players`).
    pub distinct_players: Vec<u64>,
}

/// Richer validator status — distinguishes unregistered from revoked.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ValidatorStatus {
    NotRegistered,
    Active,
    Revoked,
    RevokedForCause,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MilestoneWithValidatorStatus {
    /// Milestone record returned with validator status context.
    pub milestone: Milestone,
    /// Current status of the validator that approved the milestone.
    pub validator_status: ValidatorStatus,
}

/// A single verified milestone record
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Milestone {
    /// Unique player identifier this milestone belongs to.
    pub player_id: u64,
    /// Validator wallet that approved the milestone.
    pub validator: Address,
    /// Human-readable milestone description.
    pub description: String,
    /// IPFS/Arweave CID of supporting evidence (video clip, stat sheet, etc.)
    pub evidence_hash: String,
    /// Ledger timestamp when the milestone was approved, in Unix seconds.
    pub approved_at: u64,
    /// Stellar ledger sequence at time of approval for tamper-proof auditability
    pub ledger_sequence: u32,
}

/// Validator entry in the trusted registry
#[contracttype]
#[derive(Clone, Debug)]
pub struct Validator {
    /// Validator wallet authorized to approve milestones.
    pub wallet: Address,
    /// Human-readable credential label (e.g. "UEFA B License", "Academy Director")
    pub credentials: String,
    /// Admin-verified organization the validator represents.
    pub affiliation: String,
    pub registered_at: u64,
    /// Whether this validator is currently authorized to approve milestones.
    pub active: bool,
    /// Optional specialization tags (e.g. "physical-stats", "identity-kyc", "match-performance").
    /// When a milestone category is provided to `approve_milestone`, only validators with a
    /// matching specialization tag can approve it. An empty Vec means the validator can approve
    /// any untagged (general-category) milestone but cannot approve tagged milestones.
    pub specializations: Vec<String>,
}

/// Entry in the global milestone index for on-chain auditability.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GlobalMilestoneEntry {
    /// Unique player identifier for the indexed milestone.
    pub player_id: u64,
    /// Per-player milestone index for fetching the full milestone.
    pub milestone_index: u32,
}

/// Paginated response for global milestone index queries.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GlobalMilestoneIndexPage {
    /// Page of global milestone index entries.
    pub entries: Vec<GlobalMilestoneEntry>,
    /// Total number of milestones in the global index.
    pub total: u32,
}

/// A player-initiated dispute for a milestone.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MilestoneDispute {
    /// Unique player identifier for the disputed milestone.
    pub player_id: u64,
    /// Per-player milestone index being disputed.
    pub milestone_index: u32,
    /// Player-provided dispute reason.
    pub reason: String,
    /// Ledger timestamp when the dispute was opened, in Unix seconds.
    pub disputed_at: u64,
    /// Whether the dispute has been resolved.
    pub resolved: bool,
    /// Whether the dispute was upheld when resolved.
    pub upheld: bool,
}

/// A lightweight reference to a milestone (player + index).
/// Stored in `DataKey::ValidatorMilestones` as a compact per-validator index.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MilestoneRef {
    /// Unique player identifier for the referenced milestone.
    pub player_id: u64,
    /// Per-player milestone index.
    pub milestone_index: u32,
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
    /// Proposed replacement admin awaiting acceptance by that address.
    PendingAdmin,
    Initialized,
    Paused,
    /// Function-scoped pause flag for approve_milestone (independent of whole-contract Paused)
    PausedApproveMilestone,
    ProgressContract,
    ProgressContractSet,
    Validator(Address),
    MilestoneCounter(u64),
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
