# ScoutChain Glossary

Domain-specific terms used throughout the contracts, documentation, and SDKs.
Each definition includes a role description and links to the relevant contract
functions in [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md).

---

## Attestation

An independent vote by one validator toward a **k-of-n threshold** milestone
claim, cast via `attest_milestone`. This is the alternate, multi-validator
commit path alongside the original single-validator `approve_milestone` path
(see [Milestone](#milestone)): once the admin calls
`set_milestone_threshold(n)` with `n >= 2`, `approve_milestone` (and the
off-chain-signed relay `submit_attested_milestone`) stop working, and a
milestone claim — identified by `(player_id, evidence_hash)`, not by its
description — only commits once `threshold` distinct active validators have
each independently attested to it within the current voting round. A vote
arriving after `voting_window_secs` has elapsed since the round started
starts a fresh round instead of counting toward the old tally. The default
threshold is `1`, which reproduces `approve_milestone`'s original
single-signature behaviour unchanged, so existing integrations keep working
until an operator deliberately opts into k-of-n mode.

- See [CONTRACT_REFERENCE.md — k-of-n threshold milestone
  attestation](CONTRACT_REFERENCE.md#k-of-n-threshold-milestone-attestation)
  for the full design rationale (claim identity, bounded storage, revoke-mid-vote
  and window-expiry semantics).
- Relevant functions: `attest_milestone`, `submit_attested_milestone`,
  `set_milestone_threshold`, `get_milestone_threshold`, `get_pending_claim`,
  `has_attested` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#verification).

---

## CID (Content Identifier)

A self-describing content hash produced by IPFS or Arweave. CIDs are stored
on-chain as strings inside player profiles and milestone evidence fields so
that off-chain video and photo assets can be retrieved and verified without
trusting a centralised server.

- IPFS CIDs start with `Qm…` (CIDv0) or `bafy…` (CIDv1).
- The `evidence_hash` parameter of `approve_milestone` and the `details_hash`
  parameter of `log_trial_offer` both accept CIDs.
- Relevant functions: `register_player`, `update_profile`, `approve_milestone`,
  `log_trial_offer` — see [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md).

---

## Contact Fee

A micro-payment in XLM (denominated in stroops) that a scout pays to unlock a
specific player's full contact details. Controlled by the `contact_fee_stroops`
field in [`FeeConfig`](#feeconfig).

- Relevant function: [`pay_to_contact`](CONTRACT_REFERENCE.md#pay_to_contactscout-address-player_id-u64---resultscoutaccesserror).

---

## ContactRecord

A record of a paid contact attempt from a scout to a player, stored by the
`scout_access` contract after successful payment of the configured contact fee.
A `ContactRecord` links the paying scout, the contacted player, and the paid
fee amount, and enables the platform to enforce repeated-contact and contact
history checks.

- Relevant functions: `pay_to_contact`, `get_contact_record`, `has_contacted`
  — see [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#scout_access).

---

## FeeConfig

The primary configuration struct for the `scout_access` contract. Controls all
subscription and pay-to-contact fee rates. Set at `initialize` time and
adjustable via `update_fee_config`.

| Field | Type | Unit | Valid Range | Typical Value |
|---|---|---|---|---|
| `contact_fee_stroops` | `i128` | stroops (1 XLM = 10 000 000 stroops) | > 0 | `100000` (0.01 XLM) |
| `basic_sub_stroops` | `i128` | stroops | > 0 | `1000000` (0.1 XLM) |
| `pro_sub_stroops` | `i128` | stroops | > 0 | `3000000` (0.3 XLM) |
| `elite_sub_stroops` | `i128` | stroops | > 0 | `7000000` (0.7 XLM) |
| `sub_duration_secs` | `u64` | duration in seconds (not a Unix timestamp) | > 0 | `2592000` (30 days) |
| `pro_contact_limit` | `u32` | count | > 0 | `10` (10 contacts/period) |
| `trial_offer_escrow_stroops` | `i128` | stroops | > 0 | `500000` (0.05 XLM) |
| `trial_offer_expiry_secs` | `u64` | duration in seconds | > 0 | `3600` (1 hour) |

All fields must be strictly greater than zero; `initialize` and
`update_fee_config` return `InvalidInput` otherwise.

`pro_contact_limit` caps the number of unique players a **Pro-tier** scout
may contact in a single subscription period. Reaching the limit causes
`pay_to_contact` to return `ProContactLimitReached` (code 20). **Elite-tier
scouts are exempt** from this cap.

`trial_offer_escrow_stroops` is the XLM amount held in escrow when a scout
logs a trial offer via `log_trial_offer`. The escrowed amount is released to
the contract's accumulated fees on successful `confirm_trial_offer`, or
refunded to the originating scout if the offer expires (confirmed after
`trial_offer_expiry_secs` have elapsed, or swept by `expire_trial_offers`).

`trial_offer_expiry_secs` defines the window (in seconds) within which a
player must call `confirm_trial_offer` after the offer was logged. After this
window the confirmation path refunds the scout's escrow and emits
`trial_offer_expired`.

- Relevant functions: `initialize`, `update_fee_config`, `get_fee_config` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#scout_access).

---

## Milestone

A verified player achievement recorded on-chain by an authorised validator.
Each milestone stores a plain-text description, an IPFS/Arweave evidence CID,
the approving validator's address, and a ledger sequence number for
auditability.

Examples: "Scored 5 goals in Local Cup", "Top speed clocked at 32 km/h".

A milestone commits through one of two paths: the original single-validator
`approve_milestone` (one signature is enough), or, once the admin opts into
k-of-n mode via `set_milestone_threshold(n >= 2)`, the multi-validator
[Attestation](#attestation) path (`attest_milestone`), which requires several
independent validators to corroborate the same claim before it commits. Only
one path is active at a time — see [Attestation](#attestation) for the
mechanics.

- Relevant functions: `approve_milestone`, `attest_milestone`,
  `get_milestone`, `get_milestone_count` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#verification).

---

## Milestone Dispute

A formal on-chain challenge raised by a player against a specific milestone
that was approved for their profile. Only the affected player may file a
dispute — validators and scouts have no standing to do so.

A dispute record carries two outcome fields that are set when the platform
admin resolves it:

| Field | Values | Meaning |
|---|---|---|
| `resolved` | `false` / `true` | Whether the admin has acted on the dispute |
| `upheld` | `false` / `true` | `true` if the admin agreed the milestone was invalid; `false` if the milestone stands |

When a dispute is upheld the admin is expected to revoke or correct the
offending milestone through the standard validator-management flow; the
dispute mechanism itself only records the outcome on-chain.

- Relevant functions: `dispute_milestone`, `resolve_dispute`, `get_dispute`,
  `has_dispute` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#verification).

---

## Player

A registered footballer with an on-chain identity. A player is identified by a
`player_id` (auto-incremented `u64`) and a Stellar wallet address. Players
start at `ProgressLevel` 0 (Unverified) and advance through up to four levels
as validators approve milestones and scouts log trial offers.

- Relevant functions: `register_player`, `get_player`, `filter_players` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#registration).

---

## Progress Level

The four-tier trust ranking attached to every player profile. Levels advance
sequentially; skipping or reversing is blocked by the progress contract (admin
`reset_player_level` is the only exception).

| Level | Variant | Meaning |
|---|---|---|
| 0 | `Unverified` | Profile created, no verifications |
| 1 | `VerifiedIdentity` | Identity confirmed by a validator |
| 2 | `PerformanceMilestones` | Performance stats verified by a validator |
| 3 | `EliteTier` | Trial offer logged by an Elite-tier scout |

- Relevant functions: `advance_level`, `get_level`, `get_progress_history`,
  `reset_player_level` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#progress).

---

## Scout

A talent-discovery professional registered on-chain with a Stellar wallet.
Scouts purchase a subscription tier (`Basic`, `Pro`, or `Elite`) to access the
filtered player pool, pay per-contact fees to unlock player details, and (Elite
only) log trial offers that advance a player to Level 3.

- Relevant functions: `register_scout`, `subscribe`, `pay_to_contact`,
  `log_trial_offer` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#registration).

---

## Stroop

The smallest unit of XLM. 1 XLM = 10 000 000 stroops. All fee fields in
`FeeConfig` and all fee-related return values in the `scout_access` contract
are expressed in stroops (Rust type `i128`).

Worked example: the documented `contact_fee_stroops` value `100000` equals
0.01 XLM (`100000 / 10 000 000`). Keep fee amounts in stroops when comparing
or tuning cost-sensitive calls such as `subscribe`, `pay_to_contact`, and
`batch_contact_players`; their CPU guardrails are tracked in
[`ci/cpu-cost-budget.md`](../ci/cpu-cost-budget.md). If a fee is too low or fee
arithmetic exceeds safe bounds, see the `scout_access` [`InsufficientFee` and
`Overflow` error codes](CONTRACT_REFERENCE.md#scoutaccesserror-scout_access-contract).

---

## Subscription Tier

The access level purchased by a scout. Determines which players are visible and
whether trial offers can be logged.

| Tier | Variant | Notes |
|---|---|---|
| Basic | `Basic` | Access to the filtered player pool |
| Pro | `Pro` | Higher trust signal; wider discovery |
| Elite | `Elite` | Required to call `log_trial_offer` |

Subscriptions expire after `sub_duration_secs` (default 30 days). Downgrades
while a subscription is active are blocked; upgrades charge the full new-tier
fee with no proration.

- Relevant functions: `subscribe`, `get_subscription` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#scout_access).

---

## Timestamp

All absolute on-chain timestamps in this project are Unix seconds: the number
of seconds elapsed since 1970-01-01 00:00:00 UTC, obtained from the Soroban
ledger timestamp. This applies to fields such as `registered_at`, `updated_at`,
`approved_at`, `disputed_at`, `expires_at`, `subscribed_at`, `contacted_at`,
`logged_at`, and `period_start`, as well as the `since_timestamp` parameter of
`get_history_since`.

`ledger_sequence` is not a timestamp; it is the Soroban ledger sequence number
recorded alongside an event. `sub_duration_secs` is a duration in seconds, not
an absolute Unix timestamp.

Example: a `ProgressEntry` might record `updated_at: 1_735_689_600` and
`ledger_sequence: 12_345_678` for the same level change. The first value is a
Unix-second wall-clock time; the second is the Soroban ledger number that
included the change.

---

## Trial Offer

An on-chain record that a scout has offered a player a trial or professional
opportunity. Logging a trial offer also advances the player to `EliteTier`
(Level 3) via a cross-contract call to the progress contract. Only scouts with
an active Elite subscription may log trial offers.

- Relevant functions: `log_trial_offer`, `get_trial_offer`, `get_trial_count`
  — see [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#scout_access).

---

## Validator

A trusted third party (local coach, academy director, or certified trainer)
registered by the platform admin. Only active validators may call
`approve_milestone`. A validator can be revoked by the admin; revoked validators
cannot approve further milestones until re-activated. If a validator is revoked
for cause (e.g. misconduct), their past milestones are flagged so they can be
weighed appropriately by scouts and indexers.

A validator may also carry **specialization tags** (e.g. `"physical-stats"`,
`"identity-kyc"`, `"match-performance"`), set via
`set_validator_specializations`. When `approve_milestone` is called with a
non-`None` `milestone_category`, the contract requires the approving
validator to hold a matching tag, rejecting the call with
`SpecializationMismatch` (code 21) otherwise — preventing, for example, a
pure identity-KYC agent from approving physical performance data. This is
backward-compatible: a validator with no specializations (the default) is
general-purpose, and an **omitted** `milestone_category` remains open to any
active validator regardless of tags.

- Relevant functions: `register_validator`, `revoke_validator`,
  `set_validator_specializations`, `get_validator_status`, `approve_milestone`,
  `get_milestone_with_validator_status` — see
  [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md#verification).
