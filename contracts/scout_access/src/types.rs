use soroban_sdk::{contracttype, Address, String};

pub use scoutchain_shared_types::WiringLink;

/// Subscription tier for scouts
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum SubscriptionTier {
    /// Basic — browse verified players (Level 1+)
    Basic,
    /// Pro — browse all levels + contact up to 10 players/month
    Pro,
    /// Elite — unlimited contacts + trial offer logging
    Elite,
}

/// Active scout subscription record
#[contracttype]
#[derive(Clone, Debug)]
pub struct Subscription {
    /// Scout wallet that owns this subscription.
    pub scout: Address,
    /// Active subscription tier for authorization and fee checks.
    pub tier: SubscriptionTier,
    /// Ledger timestamp when the subscription expires, in Unix seconds.
    pub expires_at: u64,
    /// Ledger timestamp when the subscription started, in Unix seconds.
    pub subscribed_at: u64,
}

/// A recorded contact event from a scout to a player
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContactRecord {
    /// Player identifier that the scout contacted.
    pub player_id: u64,
    /// Scout wallet that initiated the contact.
    pub scout: Address,
    /// Ledger timestamp at the moment the contact was recorded
    pub contacted_at: u64,
}

/// A logged trial offer from a scout to a player
#[contracttype]
#[derive(Clone, Debug)]
pub struct TrialOffer {
    /// Player identifier receiving the trial offer.
    pub player_id: u64,
    /// Scout wallet that logged the trial offer.
    pub scout: Address,
    /// IPFS/Arweave CID of the offer details document
    pub details_hash: String,
    /// Ledger timestamp when the trial offer was logged, in Unix seconds.
    pub logged_at: u64,
}

/// Tracks the number of contacts a Pro-tier scout has made in their current
/// subscription period.  `period_start` is the `subscribed_at` timestamp of
/// the current subscription; when the scout renews, a new record is stored
/// (keyed by the new `subscribed_at`), effectively resetting the counter.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProContactPeriod {
    /// `subscribed_at` of the subscription this counter belongs to.
    /// Used to detect period rollovers on subscription renewal.
    pub period_start: u64,
    /// Number of contacts made in this period.
    pub count: u32,
}

/// Escrow record for a trial offer
#[contracttype]
#[derive(Clone, Debug)]
pub struct TrialEscrow {
    /// Escrowed trial-offer amount in stroops, set from
    /// `FeeConfig.trial_offer_escrow_stroops` when `log_trial_offer` creates
    /// the escrow.
    pub amount: i128,
    /// Ledger timestamp deadline, in Unix seconds, computed as the current
    /// ledger timestamp plus `FeeConfig.trial_offer_expiry_secs`; checked by
    /// `confirm_trial_offer` when deciding whether the trial offer is still
    /// valid.
    pub expires_at: u64,
}

/// Proposed fee configuration awaiting activation after a delay
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeConfigProposal {
    /// The proposed fee configuration
    pub config: FeeConfig,
    /// Ledger timestamp when the proposal was created
    pub proposed_at: u64,
}

/// Platform fee configuration
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeConfig {
    /// Contact fee in stroops (1 XLM = 10_000_000 stroops)
    pub contact_fee_stroops: i128,
    /// Basic subscription fee in stroops
    pub basic_sub_stroops: i128,
    /// Pro subscription fee in stroops
    pub pro_sub_stroops: i128,
    /// Elite subscription fee in stroops
    pub elite_sub_stroops: i128,
    /// Subscription duration in seconds (default: 30 days)
    pub sub_duration_secs: u64,
    /// Trial offer escrow hold amount in stroops.
    /// Must be > 0 when trial offers are enabled; 0 disables trial offers.
    pub trial_offer_escrow_stroops: i128,
    /// Trial offer expiry window in seconds.
    /// Must be > 0; defines how long an escrowed trial offer remains valid.
    pub trial_offer_expiry_secs: u64,
    /// Maximum contacts per month for Pro tier (default: 10)
    /// Elite-tier scouts are exempt from this cap (no limit applies).
    /// See `docs/CONTRACT_REFERENCE.md` — `FeeConfig` and `ProContactLimitReached`
    /// (error 20) for the full per-tier access semantics, and `docs/GLOSSARY.md`
    /// for the definition of "Pro tier" and the contact quota model.
    pub pro_contact_limit: u32,
}

/// A single entry in the bounded on-chain fee configuration history.
/// Stored in `DataKey::FeeConfigHistory` as a `Vec<FeeConfigHistoryEntry>`,
/// oldest-first, capped at `FEE_CONFIG_HISTORY_CAP` entries.
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeConfigHistoryEntry {
    /// The fee configuration that was active before this change.
    pub config: FeeConfig,
    /// Ledger timestamp (Unix seconds) when this config was set via `update_fee_config`.
    pub updated_at: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    /// Proposed replacement admin awaiting acceptance by that address.
    PendingAdmin,
    Initialized,
    Paused,
    FeeConfig,
    /// Proposed fee configuration awaiting activation after a 7-day delay
    PendingFeeConfig,
    AccumulatedFees,
    /// Native XLM token contract address
    XlmToken,
    /// scout wallet → Subscription
    Subscription(Address),
    /// (player_id, scout) → bool (has contacted)
    ContactRecord(u64, Address),
    /// scout → Vec<u64> of contacted player_ids
    ScoutContacts(Address),
    /// Monthly contact count for Pro tier: (scout, month_bucket) → count
    ContactCount(Address, u64),
    /// trial offer counter per player
    TrialCounter(u64),
    /// (player_id, trial_index) → TrialOffer
    TrialOffer(u64, u32),
    /// progress contract address for cross-contract advance_level call
    ProgressContract,
    /// registration contract address for cross-contract scout verification checks
    RegistrationContract,
    /// (scout, player_id) → u64 timestamp of the last trial offer sent
    /// Used to enforce the per-(scout, player) cooldown window.
    TrialOfferLastSent(Address, u64),
    /// tier → Vec<Address> of scouts subscribed at this tier
    TierSubscribers(SubscriptionTier),
    /// Pro-tier contact period counter: scout → ProContactPeriod
    ProContactCount(Address),
    /// player_id → Vec<Address> of scouts who have contacted this player
    PlayerContacts(u64),
    /// scout → Vec<(player_id, trial_index)> of all trial offers sent
    ScoutTrialOffers(Address),
    /// (player_id, trial_index) → TrialEscrow (holds escrow amount & expiry)
    TrialEscrow(u64, u32),
    /// Global Vec<(player_id, trial_index)> of TrialEscrow records that have
    /// not yet been confirmed or refunded. Maintained by `log_trial_offer`
    /// (push on creation) and `confirm_trial_offer` (remove on cleanup) so
    /// `expire_trial_offers` can sweep stale escrows without an off-chain index.
    OutstandingTrialEscrows,
    /// Bounded on-chain history of the last N FeeConfig values, oldest-first.
    /// Updated by `update_fee_config`. Exposed via `get_fee_config_history`.
    FeeConfigHistory,
    /// Idempotency nonce for `confirm_trial_offer` retries.
    /// Maps caller-supplied nonce → () so a retried call after a
    /// `ProgressCallFailed` can safely detect that the offer was already
    /// confirmed and skip the escrow cleanup / level-advance replay.
    ConfirmationNonce(String),
    /// scout wallet → bool; true if the scout has opted in to auto-renewal.
    /// Set by `set_auto_renew`, consumed by `renew_if_due`.
    AutoRenew(Address),
    /// Day-granularity expiry bucket: (expires_at / 86_400) → Vec<Address>.
    ///
    /// Maintained by `subscribe` alongside `Subscription(scout)` so that
    /// `get_expiring_subscriptions` can page through soon-to-expire
    /// subscriptions in O(days_covered) without walking every scout.
    ///
    /// Tradeoff: coarse day-bucket granularity keeps index storage cost low
    /// (one Vec per day with at least one subscriber) at the cost of requiring
    /// the caller to re-check `Subscription.expires_at` for exact filtering,
    /// which `get_subscriptions_expiring_before` already does.
    ExpiryBucket(u64),
    /// Re-wiring epoch for `DataKey::ProgressContract`, bumped by every
    /// `set_progress_contract` / `update_progress_contract` call. See
    /// `scoutchain_shared_types::WiringLink` and
    /// `docs/WIRING_REGISTRY_DESIGN.md` (issue #1041).
    ProgressContractEpoch,
    /// Re-wiring epoch for `DataKey::RegistrationContract`, bumped by every
    /// `set_registration_contract` call.
    RegistrationContractEpoch,
}

/// Snapshot of both cross-contract peer address pointers held by the
/// scout_access contract, each with its address and re-wiring epoch.
/// Returned by `ScoutAccessContract::get_wiring_state`. See
/// `docs/WIRING_REGISTRY_DESIGN.md` for the full cross-contract picture.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ScoutAccessWiringState {
    /// Peer link to the progress contract, set via `set_progress_contract`
    /// (or its `update_progress_contract` alias). Used so `log_trial_offer`
    /// can atomically advance a player to Level 3.
    pub progress_contract: WiringLink,
    /// Peer link to the registration contract, set via
    /// `set_registration_contract`. Used for Pro-tier scout verification
    /// gating.
    pub registration_contract: WiringLink,
}

impl ScoutAccessWiringState {
    /// Returns `true` iff both peer links are configured.
    pub fn is_fully_wired(&self) -> bool {
        self.progress_contract.is_configured() && self.registration_contract.is_configured()
    }
}
