pub use scoutchain_shared_types::{ContractHealth, WiringLink};
use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

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
    /// Administrator-verified organizational affiliation (e.g. "FC Example Academy")
    pub affiliation: String,
    /// Ledger timestamp when the validator was registered, in Unix seconds.
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

/// Off-chain signed milestone attestation (issue #703).
///
/// Canonical signed message (domain-separated):
/// `ATTESTATION_DOMAIN || contract_id || network_id || validator_wallet
///  || player_id_be || description_bytes || evidence_hash_bytes || nonce_be`
///
/// Field rationale:
/// - `validator_wallet`: binds the claim to a registry identity; after signature
///   verification against that wallet's registered pubkey, this is the sole
///   source of attribution (never a separate caller-supplied Address).
/// - `player_id` / `description` / `evidence_hash`: exact claim being attested.
/// - `nonce`: strictly-increasing per-validator counter for replay protection
///   (raw ed25519 signatures have no Soroban sequence number).
/// - `contract_id` + `network_id`: prevent cross-deployment / cross-network replay.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MilestoneAttestation {
    /// Validator whose registered attestation key must have signed this payload.
    pub validator_wallet: Address,
    /// Player receiving the milestone.
    pub player_id: u64,
    /// Human-readable milestone description.
    pub description: String,
    /// IPFS/Arweave CID of supporting evidence.
    pub evidence_hash: String,
    /// Strictly increasing per-validator nonce (must be > last accepted).
    pub nonce: u64,
    /// Must equal `env.current_contract_address()` at verification time.
    pub contract_id: Address,
    /// Must equal `env.ledger().network_id()` at verification time.
    pub network_id: BytesN<32>,
}

/// Bounded, fixed-size accumulator for a k-of-n milestone attestation claim
/// (issue: threshold milestone approval). Keyed by canonical claim identity
/// (player_id, evidence_hash) — see `attest_milestone` for why description
/// text is intentionally excluded from the identity.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct PendingMilestoneClaim {
    pub player_id: u64,
    pub evidence_hash: String,
    /// Description locked in by the first attestation of this (player_id,
    /// evidence_hash, round). Later voters' description text does not
    /// overwrite it, so the threshold-reaching validator cannot rewrite the
    /// claim's narrative at the last moment.
    pub description: String,
    /// Distinct, currently-valid active-validator votes accumulated so far
    /// in this round.
    pub vote_count: u32,
    /// Bumped on every voting-window expiry; invalidates all prior votes
    /// without touching their storage — see `DataKey::PendingMilestoneVote`.
    pub round: u32,
    /// Ledger timestamp (Unix seconds) this round started.
    pub created_at: u64,
    /// Threshold snapshotted when this round started, so an admin changing
    /// the global threshold mid-vote cannot retroactively fast-track or
    /// invalidate an in-flight claim.
    pub threshold: u32,
}

/// Reference to one of a validator's currently-open pending-claim votes.
/// Stored (bounded, capped) under `DataKey::ValidatorPendingVotes` purely so
/// `revoke_validator` can find and retract this validator's contribution to
/// any still-pending claim without an unbounded storage scan.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct PendingVoteRef {
    pub player_id: u64,
    pub evidence_hash: String,
    pub round: u32,
}

/// Result of `attest_milestone` — whether this vote just crossed threshold.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum AttestationStatus {
    /// Vote recorded; still short of threshold. Payload is the new vote count.
    Pending(u32),
    /// This vote reached threshold; the milestone was committed and
    /// `progress.advance_level` was invoked. Payload is the milestone index.
    Committed(u32),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct DiversityConfig {
    pub required_distinct_affiliations: u32,
    pub starting_milestone_index: u32,
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
    ValidatorMilestoneCount(Address),
    ValidatorPlayerMilestoneCount(Address, u64),
    ValidatorVector,
    TotalMilestoneCount,
    GlobalMilestoneIndex,
    /// Persistent config for diversity-gated milestone advancement
    DiversityConfig,
    /// Persistent index: player_id → Vec<String> distinct affiliations that have contributed milestones
    PlayerAffiliations(u64),
    /// Persistent index: validator wallet → Vec<u64> of distinct player_ids
    /// for which that validator has approved at least one milestone.
    /// Updated on every `approve_milestone` call (duplicates are skipped).
    ValidatorPlayers(Address),
    MilestoneDispute(u64, u32),
    ActiveValidatorCount,
    TotalValidatorCount,
    /// Evidence hash → (player_id, milestone_index) for global uniqueness and usage lookup.
    EvidenceUsed(String),
    ValidatorMilestones(Address),
    ActiveDisputesCount,
    ValidatorRevokedForCause(Address),
    /// Per-player list of milestone indices that have been disputed.
    /// player_id → Vec<u32> of milestone_index values.
    /// Updated on `dispute_milestone`.
    PlayerDisputes(u64),
    /// Persistent global index of currently-unresolved (player_id, milestone_index) pairs.
    /// Populated on `dispute_milestone`, pruned on `resolve_dispute`.
    /// Exposed via `list_disputes_page(offset, limit)`.
    OpenDisputeIndex,

    // ── Registration cooldown ──
    /// Last registration timestamp for a validator wallet (Unix seconds).
    /// Set by `register_validator` and read to enforce the per-caller cooldown.
    ValidatorRegLastSent(Address),
    /// Platform-wide validator registration cooldown in seconds.
    /// 0 disables the cooldown. Configurable by admin via `set_reg_cooldown`.
    /// The `u64` payload is unused (always written as `RegCooldownSecs(0)`).
    RegCooldownSecs(u64),
    /// Minimum distinct validator regions required for Level-2/3 advances.
    MinRegionQuorum,
    /// Validator wallet → registered ed25519 public key (32 bytes) for
    /// off-chain milestone attestation verification.
    AttestationKey(Address),
    /// Reverse index: attestation pubkey → validator wallet. Used so identity
    /// is derived from the verified key, not from a caller-supplied Address.
    AttestationKeyOwner(BytesN<32>),
    /// Per-validator monotonic nonce for relayed attestation replay protection.
    /// Stores the last successfully consumed nonce (starts absent → treat as 0).
    AttestationNonce(Address),

    // ── k-of-n threshold milestone attestation ──
    /// Pending (sub-threshold) milestone attestation accumulator, keyed by
    /// the canonical claim identity (player_id, evidence_hash). See
    /// `attest_milestone`.
    PendingMilestoneClaim(u64, String),
    /// One validator's vote on a specific (player_id, evidence_hash, round).
    /// `round` is bumped whenever a sub-threshold claim expires, which makes
    /// every vote cast in a prior round unreachable without needing to
    /// delete or enumerate it — see `attest_milestone` for the expiry
    /// mechanism.
    PendingMilestoneVote(u64, String, u32, Address),
    /// Bounded list (capped at MAX_PENDING_VOTES_PER_VALIDATOR) of claims a
    /// validator currently has an open, uncommitted, unexpired vote on. Used
    /// solely so `revoke_validator` can retroactively invalidate that
    /// validator's contribution to any still-pending claim without an
    /// unbounded storage scan.
    ValidatorPendingVotes(Address),
    /// k-of-n distinct-active-validator threshold required before a
    /// milestone claim accumulated via `attest_milestone` is committed.
    /// Defaults to 1 — see `get_milestone_threshold`.
    MilestoneApprovalThreshold,
    /// Voting window (seconds) within which `threshold` distinct votes must
    /// accumulate before a claim expires. See `get_voting_window_secs`.
    AttestationVotingWindowSecs,

    // ── Registration cross-contract (issue #1014) ──
    /// Address of the registration contract used to verify wallet↔player_id binding.
    RegistrationContract,
    /// Whether `RegistrationContract` has been set at least once.
    RegistrationContractSet,

    // ── Wiring epochs (issue #1041) ──
    /// Re-wiring epoch for `DataKey::ProgressContract`, bumped by every
    /// `set_progress_contract` / `update_progress_contract` call. See
    /// `scoutchain_shared_types::WiringLink` and
    /// `docs/WIRING_REGISTRY_DESIGN.md`.
    ProgressContractEpoch,
    /// Re-wiring epoch for `DataKey::RegistrationContract`, bumped by every
    /// `set_registration_contract` / `update_registration_contract` call.
    RegistrationContractEpoch,
}

/// Snapshot of both cross-contract peer address pointers held by the
/// verification contract, each with its address and re-wiring epoch.
/// Returned by [`VerificationContract::get_wiring_state`].
///
/// See `docs/WIRING_REGISTRY_DESIGN.md` for the full cross-contract picture
/// and `scoutchain_shared_types::WiringLink` for what `epoch` means.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VerificationWiringState {
    /// Peer link to the progress contract. Set via `set_progress_contract`
    /// (first call only — see `DataKey::ProgressContractSet`) or
    /// `update_progress_contract` (any subsequent call). Only this address
    /// may be the target of `advance_level` cross-calls from
    /// `approve_milestone` / `attest_milestone`.
    pub progress_contract: WiringLink,
    /// Peer link to the registration contract. Set via
    /// `set_registration_contract` (first call only — see
    /// `DataKey::RegistrationContractSet`) or `update_registration_contract`.
    /// Used by `dispute_milestone` to verify wallet↔player_id binding.
    pub registration_contract: WiringLink,
}

impl VerificationWiringState {
    /// Returns `true` iff both peer links are configured.
    pub fn is_fully_wired(&self) -> bool {
        self.progress_contract.is_configured() && self.registration_contract.is_configured()
    }
}
