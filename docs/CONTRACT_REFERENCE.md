# Contract Reference

Complete public API reference for all four ScoutChain Soroban smart contracts.
Every `pub fn` in every `#[contractimpl]` block is documented here.

> [!NOTE]
> **Last verified:** 2026-07-26 — manually cross-checked against the contract source and the H2/Table of Contents audit in this documentation-sync PR.

---

All `stellar contract invoke` examples below pass `String` and enum arguments
as JSON values wrapped in shell single quotes, for example `--tier '"Elite"'`.
That keeps the command copy-paste-runnable in a standard `bash`/`zsh` shell.

## Table of Contents

- [registration](#registration)
- [verification](#verification)
- [progress](#progress)
- [scout_access](#scout_access)
- [Shared Types](#shared-types)
  - [`ProgressLevel`](#progresslevel)
  - [`ContractHealth`](#contracthealth)
  - [`PlayerVitals`](#playervitals)
  - [`PlayerProfile`](#playerprofile)
  - [`ScoutProfile`](#scoutprofile)
  - [`Validator`](#validator)
  - [`ValidatorStatus`](#validatorstatus)
  - [`Milestone`](#milestone)
  - [`MilestoneDispute`](#milestonedispute)
  - [`ProgressEntry`](#progressentry)
  - [`SubscriptionTier`](#subscriptiontier)
  - [`Subscription`](#subscription)
  - [`ContactRecord`](#contactrecord)
  - [`FeeConfig`](#feeconfig)
  - [`ProContactPeriod`](#procontactperiod)
  - [`TrialOffer`](#trialoffer)
- [Error Codes](#error-codes)
- [Events](#events)
- [Design Discussion: Check-Ordering Follow-ups](#design-discussion-check-ordering-follow-ups)

---

## registration

Handles player and scout on-chain identity: registration, profile updates,
deregistration, and discovery queries.

Timestamp fields returned by this contract (`registered_at` and `updated_at`)
are Unix seconds. See [Timestamp](GLOSSARY.md#timestamp).

### Functions

---

#### `initialize(admin: Address) -> Result<(), ScoutChainError>`

One-time contract setup. Must be called before any other function.

| | |
|---|---|
| **Auth** | `admin` must sign |
| **Errors** | `AlreadyInitialized` if called more than once |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- initialize --admin $ADMIN_ADDRESS
```

---

#### `propose_admin(new_admin: Address) -> Result<(), ScoutChainError>`

Store or replace a pending admin proposal. The current admin retains all
privileges until the proposed address accepts.

| | |
|---|---|
| **Auth** | Current admin must sign |
| **Errors** | `NotInitialized` |
| **Emits** | `admin_transfer_proposed` with `(old_admin, new_admin)` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- propose_admin --new_admin $NEW_ADMIN_ADDRESS
```

---

#### `accept_admin() -> Result<(), ScoutChainError>`

Finalize the pending transfer. The stored pending admin must sign, proving
control of the address. Acceptance updates the admin and clears the proposal.

| | |
|---|---|
| **Auth** | Pending admin must sign |
| **Errors** | `NotInitialized` · `PendingAdminNotSet` |
| **Emits** | `admin_transferred` with `(old_admin, new_admin)` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID -- accept_admin
```

---

#### `transfer_admin(new_admin: Address) -> Result<(), ScoutChainError>`

Deprecated compatibility alias for `propose_admin`. It does not immediately
change the admin; the proposed address must still call `accept_admin`.

---

#### `register_player(wallet: Address, vitals: PlayerVitals, ipfs_hashes: Vec<String>) -> Result<u64, ScoutChainError>`

Create a new on-chain player profile at Level 0 (Unverified).
Returns the assigned `player_id`.

| | |
|---|---|
| **Auth** | `wallet` must sign |
| **Errors** | `AlreadyRegistered` · `InvalidInput` (field too long or bad hash count) · `NotInitialized` · `ContractPaused` · `Overflow` |

Constraints:
- `position` and `nationality` max 64 bytes each; `region` max 100 bytes
- `ipfs_hashes` must contain 1–10 entries
- Player vitals (`position`, `region`, `nationality`, `age`) are write-once at registration time and immutable post-registration. Length limits are strictly enforced during `register_player` and cannot be bypassed via post-registration mutation.

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- register_player \
  --wallet $PLAYER_ADDRESS \
  --vitals '{"age":20,"position":"Forward","region":"West Africa","nationality":"Ghana"}' \
  --ipfs_hashes '["QmHighlightCID"]'
```

---

#### `update_profile(player_id: u64, ipfs_hashes: Vec<String>) -> Result<(), ScoutChainError>`

Replace a player's IPFS content hashes (highlight reels, photos). Note that `update_profile` accepts only `ipfs_hashes` and does not take or modify `PlayerVitals` fields. Because player vitals are write-once at registration time and immutable post-registration, length validation runs exclusively during `register_player` and no post-registration update path exists to set or modify vitals.

| | |
|---|---|
| **Auth** | Player's wallet must sign |
| **Errors** | `PlayerNotFound` · `InvalidInput` (empty or >10 hashes) · `ContractPaused` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- update_profile \
  --player_id 1 \
  --ipfs_hashes '["QmNewCID1","QmNewCID2"]'
```

---

#### `deregister_player(player_id: u64) -> Result<(), ScoutChainError>`

Remove a player profile and all associated wallet index entries.
Implements the GDPR right-to-erasure. The `player_id` is permanently freed.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `PlayerNotFound` · `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- deregister_player --player_id 1
```

---

#### `deactivate_player(player_id: u64) -> Result<(), ScoutChainError>`

Hide a player from `filter_players` results without erasing their profile
(soft-delete). Sets the `PlayerDeactivated` flag; the player's data and
`player_id` remain intact and can be restored with `reactivate_player`.
Emits a `player_deactivated` event on success.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `PlayerNotFound` · `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- deactivate_player --player_id 1
```

---

#### `reactivate_player(player_id: u64) -> Result<(), ScoutChainError>`

Reverse a prior `deactivate_player` call. Clears the `PlayerDeactivated`
flag, making the player visible in `filter_players` results again.
Emits a `player_reactivated` event on success.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `PlayerNotFound` · `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- reactivate_player --player_id 1
```

---

#### `register_scout(wallet: Address, region: String) -> Result<u64, ScoutChainError>`

Create a new scout profile. Returns the assigned `scout_id`.
Scouts start as unverified (`verified: false`); call `verify_scout` to promote.

| | |
|---|---|
| **Auth** | `wallet` must sign |
| **Errors** | `AlreadyRegistered` · `InvalidInput` (region >128 bytes) · `NotInitialized` · `ContractPaused` · `Overflow` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- register_scout \
  --wallet $SCOUT_ADDRESS \
  --region '"West Africa"'
```

---

#### `verify_scout(scout_id: u64) -> Result<(), ScoutChainError>`

Mark a scout as verified. Verified scouts gain trust-signal visibility on the
discovery dashboard.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `ScoutNotFound` · `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- verify_scout --scout_id 1
```

---

#### `set_progress_contract(addr: Address) -> Result<(), ScoutChainError>`

Store the progress contract address so `set_player_level` may only be called
by that contract. Must be called after both contracts are deployed (admin only).

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- set_progress_contract --addr $PROGRESS_CONTRACT_ID
```

---

#### `set_player_level(player_id: u64, level: ProgressLevel) -> Result<(), ScoutChainError>`

Update a player's stored `ProgressLevel`. Only callable by the registered
progress contract address via cross-contract invocation.

| | |
|---|---|
| **Auth** | Registered progress contract must sign |
| **Errors** | `Unauthorized` (progress contract not configured or wrong caller) · `PlayerNotFound` |

_Not intended for direct invocation. Called atomically by `progress.advance_level`._

---

#### `get_player(player_id: u64) -> Result<PlayerProfile, ScoutChainError>`

Retrieve the full player profile including wallet, vitals, IPFS hashes, and
current progress level.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `PlayerNotFound` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- get_player --player_id 1
```

---

#### `get_player_by_wallet(wallet: Address) -> Result<PlayerProfile, ScoutChainError>`

Look up a player profile by their Stellar wallet address. Useful when the
`player_id` is unknown.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `PlayerNotFound` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- get_player_by_wallet --wallet $PLAYER_ADDRESS
```

---

#### `get_scout(scout_id: u64) -> Result<ScoutProfile, ScoutChainError>`

Retrieve a scout profile by ID.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `ScoutNotFound` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- get_scout --scout_id 1
```

---

#### `get_player_count() -> u64`

Return the total number of registered players. Returns `0` before the contract
is initialized.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID -- get_player_count
```

---

#### `get_scout_count() -> u64`

Return the total number of registered scouts. Returns `0` before the contract
is initialized.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID -- get_scout_count
```

---

#### `filter_players(region: String, position: String, min_level: ProgressLevel) -> Result<Vec<PlayerProfile>, ScoutChainError>`

Scout discovery query. Returns up to 50 player profiles matching the given
region, position, and minimum progress level.

Uses the composite `PlayersByLevelRegion(level, region)` index as the entry
point so only players that already satisfy the level+region criteria are loaded.
Gas cost is proportional to the number of matching players, not the total player
count. The index is maintained automatically on `register_player`,
`set_player_level`, and `deregister_player`.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `NotInitialized` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- filter_players \
  --region '"West Africa"' \
  --position '"Forward"' \
  --min_level '"Unverified"'
```

---

#### `pause_contract() -> Result<(), ScoutChainError>`

Halt all state-changing operations (circuit breaker). Read-only queries remain
available.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID -- pause_contract
```

---

#### `unpause_contract() -> Result<(), ScoutChainError>`

Resume normal operations after a pause.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID -- unpause_contract
```

---

#### `health() -> ContractHealth`

Return the contract's initialization and pause status.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID -- health
```

---

#### `get_player_summary(player_id: u64) -> Result<PlayerSummary, ScoutChainError>`

Return a lightweight player view without IPFS hashes or wallet address.
Useful for scout discovery lists where the full profile is not needed.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `PlayerNotFound` |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- get_player_summary --player_id 1
```

---

#### `get_players(ids: Vec<u64>) -> Result<Vec<PlayerSummary>, ScoutChainError>`

Batch-fetch lightweight player summaries for up to 20 IDs in a single call.
Missing IDs are silently skipped (partial hits are returned without error).
For cost rationale behind batch-size caps, see the batch-operation entries in [`ci/cpu-cost-budget.md`](../ci/cpu-cost-budget.md), including `scout_access.batch_contact_players`.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `InvalidInput` (more than 20 IDs provided) |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- get_players --ids '[1,2,3]'
```

---

#### `version() -> String`

Return the deployed contract version string (from `Cargo.toml` at build time).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID -- version
```

---

### Dual-Role Wallet Policy

A single wallet may register as both a player and a scout. Cross-role
registration is permitted; duplicate prevention is enforced per role only.

---

## verification

Manages the trusted validator registry and milestone approvals. Cross-calls
`progress.advance_level` atomically when a milestone is approved.

Timestamp fields returned by this contract (`registered_at`, `approved_at`, and
`disputed_at`) are Unix seconds. See [Timestamp](GLOSSARY.md#timestamp).

### Functions

---

#### `initialize(admin: Address) -> Result<(), VerificationError>`

One-time contract setup.

| | |
|---|---|
| **Auth** | `admin` must sign |
| **Errors** | `AlreadyInitialized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- initialize --admin $ADMIN_ADDRESS
```

---

#### `propose_admin(new_admin: Address) -> Result<(), VerificationError>`

Store or replace a pending admin proposal. The current admin retains all
privileges until the proposed address accepts.

| | |
|---|---|
| **Auth** | Current admin must sign |
| **Errors** | `NotInitialized` |
| **Emits** | `admin_transfer_proposed` with `(old_admin, new_admin)` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- propose_admin --new_admin $NEW_ADMIN_ADDRESS
```

---

#### `accept_admin() -> Result<(), VerificationError>`

Finalize the pending transfer. The stored pending admin must sign, proving
control of the address. Acceptance updates the admin and clears the proposal.

| | |
|---|---|
| **Auth** | Pending admin must sign |
| **Errors** | `NotInitialized` · `PendingAdminNotSet` |
| **Emits** | `admin_transferred` with `(old_admin, new_admin)` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- accept_admin
```

---

#### `transfer_admin(new_admin: Address) -> Result<(), VerificationError>`

Deprecated compatibility alias for `propose_admin`. It does not immediately
change the admin; the proposed address must still call `accept_admin`.

---

#### `set_progress_contract(progress_contract: Address) -> Result<(), VerificationError>`

Wire the progress contract address so `approve_milestone` can call
`advance_level` cross-contract. Must be called once after deployment.
Returns `AlreadyConfigured` on subsequent calls — use
`update_progress_contract` for intentional re-wiring.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `AlreadyConfigured` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- set_progress_contract --progress_contract $PROGRESS_CONTRACT_ID
```

---

#### `update_progress_contract(progress_contract: Address) -> Result<(), VerificationError>`

Re-wire the progress contract address after the initial `set_progress_contract`
call. Use when redeploying or rotating the progress contract.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- update_progress_contract --progress_contract $NEW_PROGRESS_CONTRACT_ID
```

---

#### `register_validator(wallet: Address, credentials: String) -> Result<(), VerificationError>`

Onboard a new trusted validator (coach, academy director, certified trainer).
`credentials` is a human-readable label (max 256 bytes, e.g. `"UEFA B License"`).

The contract enforces a cap of **100 simultaneously registered validators**. This limit exists because all validator addresses are stored in a single persistent entry; exceeding Soroban's 64 KB per-entry limit would cause the entry to become unreadable. Raising the cap requires a contract upgrade.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `ValidatorAlreadyRegistered` · `InvalidInput` (credentials >256 bytes) · `ValidatorCapReached` (100-validator limit reached) · `NotInitialized` · `ContractPaused` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- register_validator \
  --wallet $VALIDATOR_ADDRESS \
  --credentials '"UEFA B License"'
```

---

#### `revoke_validator(wallet: Address, reason: Option<String>) -> Result<(), VerificationError>`

Deactivate a validator. Revoked validators cannot approve milestones.
`reason` is optional and capped at 128 bytes. If the reason is not exactly `"Routine"`, the validator is considered revoked for cause. This emits an additional `validator_revoked_for_cause` event and updates their status to `RevokedForCause` so off-chain indexers and `get_milestone_with_validator_status` can flag their historical milestones.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `ValidatorNotFound` · `ReasonTooLong` (reason >128 bytes) · `Unauthorized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- revoke_validator \
  --wallet $VALIDATOR_ADDRESS \
  --reason '"Misconduct"'
```

---

#### `batch_revoke_validators(wallets: Vec<Address>, reason: Option<String>) -> Result<(), VerificationError>`

Revoke multiple validators in a single atomic transaction. Applies the same
revoke logic as `revoke_validator` to each wallet in `wallets`, emitting one
`validator_revoked` event per revocation (and `validator_revoked_for_cause` if the reason is not `"Routine"`). If any wallet is not registered the
entire batch fails and no revocations are applied.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `ValidatorNotFound` · `ReasonTooLong` (reason >128 bytes) · `Unauthorized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- batch_revoke_validators \
  --wallets '["'$VALIDATOR_ADDRESS_1'","'$VALIDATOR_ADDRESS_2'"]' \
  --reason '"Season review"'
```

---

#### `restore_validator(wallet: Address) -> Result<(), VerificationError>`

Re-activate a previously revoked validator. The validator's credentials and
milestone history are preserved — only the `active` flag is flipped back to
`true`, so they can immediately approve milestones again.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `ValidatorNotFound` · `Overflow` · `Unauthorized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- restore_validator --wallet $VALIDATOR_ADDRESS
```

---

#### `transfer_validator(old_wallet: Address, new_wallet: Address) -> Result<(), VerificationError>`

Migrate a validator's identity to a new wallet address. Copies the
`Validator` record (credentials, registration timestamp, active flag) and the
per-validator milestone count to `new_wallet`, then removes `old_wallet`'s
storage entries and swaps it for `new_wallet` in the validator registry.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `ValidatorNotFound` (old_wallet not registered) · `ValidatorAlreadyRegistered` (new_wallet already registered, including the same-address case where `old_wallet == new_wallet`) · `Unauthorized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- transfer_validator \
  --old_wallet $OLD_VALIDATOR_ADDRESS \
  --new_wallet $NEW_VALIDATOR_ADDRESS
```

---

#### `approve_milestone(validator_wallet: Address, player_id: u64, description: String, evidence_hash: String) -> Result<u32, VerificationError>`

Record a verified milestone for a player. Caller must be a registered, active
validator. Evidence hash must be a valid IPFS (`Qm…`) or Arweave (`bafy…`) CID
of 2–128 bytes.

After storing the milestone this function cross-calls `progress.advance_level`
atomically so both state changes occur in the same Stellar transaction. Returns
the milestone index.

| | |
|---|---|
| **Auth** | `validator_wallet` must sign |
| **Errors** | `ContractPaused` · `ValidatorNotFound` · `ValidatorInactive` · `InvalidInput` (bad evidence hash) · `DuplicateEvidence` (evidence hash already used) · `MilestoneLimitExceeded` (5 milestones/player/validator cap) · `Overflow` · `ProgressCallFailed` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- approve_milestone \
  --validator_wallet $VALIDATOR_ADDRESS \
  --player_id 1 \
  --description '"Scored 5 goals in Local Cup"' \
  --evidence_hash '"QmEvidence123"'
```

---

#### `get_validators() -> Vec<Address>`

Return the list of all registered validator addresses (both active and revoked).
Capped at 100 addresses.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- get_validators
```

---

#### `get_validator_status(wallet: Address) -> ValidatorStatus`

Return the detailed status of a validator wallet: `Active`, `Revoked`, `RevokedForCause`, or
`NotRegistered`. Prefer this over `is_active_validator` for precise status
checks.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_validator_status --wallet $VALIDATOR_ADDRESS
```

---

#### `get_validator_statuses(wallets: Vec<Address>) -> Vec<ValidatorStatus>`

Batch-fetch the status of up to 20 validator wallets in a single call.
Returns one `ValidatorStatus` entry per input wallet **in the same order as the input**,
including `NotRegistered` for wallets that have never been registered.

**Batch-size cap**: the first 20 entries are processed; wallets beyond that are silently
ignored. Call again with the remainder for larger sets. This is consistent with the 20-item
cap used by `registration.get_players`.

**Semantics**: unlike `registration.get_players` (which silently skips missing IDs), this
function always returns one entry per input wallet — including `NotRegistered` — because
`ValidatorStatus` already has a `NotRegistered` variant that makes the unregistered case
unambiguously representable. Callers always receive exactly N results for N inputs (up to the
cap), making it impossible to confuse "skipped" with "not registered".

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_validator_statuses \
  --wallets '["$WALLET_1","$WALLET_2","$WALLET_3"]'
```

Compare with [`get_players`](#get_playersids-vecu64---resultvecplayersummary-scoutchainerror) in the registration contract for the equivalent batch-fetch pattern.

---

#### `get_validator_milestone_count(wallet: Address) -> u32`

Return the total number of milestones approved by a specific validator across
all players. Returns `0` for unregistered wallets.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_validator_milestone_count --wallet $VALIDATOR_ADDRESS
```

---

#### `get_milestone(player_id: u64, index: u32) -> Result<Milestone, VerificationError>`

Read a specific milestone record. Indices start at `1`.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `MilestoneNotFound` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_milestone --player_id 1 --index 1
```

---

#### `get_milestone_with_validator_status(player_id: u64, index: u32) -> Result<MilestoneWithValidatorStatus, VerificationError>`

Read a specific milestone record along with the current status of the validator who approved it. Useful for checking if the approving validator was later revoked for cause.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `MilestoneNotFound` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_milestone_with_validator_status --player_id 1 --index 1
```

---

#### `get_milestone_count(player_id: u64) -> u32`

Return the total number of approved milestones for a player.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_milestone_count --player_id 1
```

---

#### `get_validator(wallet: Address) -> Result<Validator, VerificationError>`

Read the full validator record including credentials, registration timestamp,
and active flag.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `ValidatorNotFound` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_validator --wallet $VALIDATOR_ADDRESS
```

---

#### `is_active_validator(wallet: Address) -> bool`

Boolean convenience check. Returns `true` only for registered, active
validators.

> **Deprecated** — use `get_validator_status` for precise `Active` / `Revoked` /
> `NotRegistered` disambiguation.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- is_active_validator --wallet $VALIDATOR_ADDRESS
```

---

#### `pause_contract() -> Result<(), VerificationError>`

Halt all state-changing operations. `approve_milestone` is blocked while paused.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- pause_contract
```

---

#### `unpause_contract() -> Result<(), VerificationError>`

Resume normal operations after a pause.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- unpause_contract
```

---

#### `health() -> ContractHealth`

Return the contract's initialization and pause status.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- health
```

---

#### `upgrade(new_wasm_hash: BytesN<32>) -> Result<(), VerificationError>`

Replace the contract WASM in-place. Persistent storage (admin, validator registry, milestones) survives the upgrade. Instance storage (initialized flag, progress contract link) is retained but should be re-verified after the call.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `Unauthorized` · `NotInitialized` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- upgrade --new_wasm_hash <NEW_WASM_HASH>
```

---

#### `get_total_milestone_count() -> u32`

Return the total number of milestones approved across all players and validators.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- get_total_milestone_count
```

---

#### `get_validator_players(wallet: Address) -> Vec<u64>`

Return all distinct player IDs for which `wallet` has approved at least one
milestone. Accumulated on every `approve_milestone` call; each player ID
appears at most once.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_validator_players --wallet $VALIDATOR_ADDRESS
```

---

#### `get_active_validator_count() -> u32`

Return the number of currently active (non-revoked) validators.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- get_active_validator_count
```

---

#### `get_validator_count() -> u32`

Return the total number of registered validators (both active and revoked).
Useful as a pre-check before calling `register_validator` to anticipate a
possible `ValidatorCapReached` error, since the validator registry is capped at
100 addresses total.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- get_validator_count
```

---

#### `get_active_disputes_count() -> u32`

Return the number of currently active (unresolved) disputes across all
players and milestones. The count is incremented on every `dispute_milestone`
call and decremented when `resolve_dispute` marks a dispute resolved.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- get_active_disputes_count
```

---

#### `get_global_milestone_index(offset: u32, limit: u32) -> GlobalMilestoneIndexPage`

Return a page of the global milestone index — a rolling log of the most
recent `(player_id, milestone_index)` pairs across all players and
validators (capped at 500 entries; oldest entries are evicted first).
`limit` is capped at 50 entries per page. `GlobalMilestoneIndexPage` has
`entries: Vec<GlobalMilestoneEntry>` and `total: u32`.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_global_milestone_index --offset 0 --limit 50
```

---

#### `get_validator_milestones(wallet: Address) -> Vec<MilestoneRef>`

Return the list of `(player_id, milestone_index)` references for every
milestone `wallet` has approved. `MilestoneRef` has `player_id: u64` and
`milestone_index: u32`. This legacy method is unbounded; high-volume callers
should use `get_validator_milestones_page` instead.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_validator_milestones --wallet $VALIDATOR_ADDRESS
```

---

#### `get_validator_milestones_page(wallet: Address, offset: u32, limit: u32) -> Vec<MilestoneRef>`

Return a bounded page of `(player_id, milestone_index)` references for milestones
approved by `wallet`. `offset` is zero-based and `limit` is capped at 50 entries,
matching `get_global_milestone_index`. Returns an empty `Vec` when the offset is
beyond the validator's approval history or `limit` is zero.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_validator_milestones_page --wallet $VALIDATOR_ADDRESS --offset 0 --limit 50
```

---

#### `dispute_milestone(player_wallet: Address, player_id: u64, milestone_index: u32, reason: String) -> Result<(), VerificationError>`

Allow a player to dispute a milestone they believe was wrongly attributed.
Only the player associated with `player_id` may submit a dispute. A new dispute
is stored as `resolved: false` and `upheld: false`. Only one dispute record may
exist per `(player_id, milestone_index)` pair. Emits a `milestone_disputed` event.

| | |
|---|---|
| **Auth** | `player_wallet` must sign |
| **Errors** | `ContractPaused` · `NotInitialized` · `MilestoneNotFound` · `Unauthorized` · `InvalidInput` (dispute already exists) |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- dispute_milestone \
  --player_wallet $PLAYER_ADDRESS \
  --player_id 1 \
  --milestone_index 1 \
  --reason '"Milestone not actually completed"'
```

---

#### `resolve_dispute(player_id: u64, milestone_index: u32, upheld: bool) -> Result<(), VerificationError>`

Admin-only review action for a filed milestone dispute. Marks the stored
`MilestoneDispute` as `resolved: true`, records the admin's outcome in `upheld`,
decrements `get_active_disputes_count()`, and emits a `dispute_resolved` event.
This function deliberately does not roll back player progress when `upheld` is
true; that corrective workflow is tracked separately.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `ContractPaused` · `NotInitialized` · `Unauthorized` · `MilestoneNotFound` (no dispute recorded) · `DisputeAlreadyResolved` |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- resolve_dispute --player_id 1 --milestone_index 1 --upheld false
```

---

#### `get_dispute(player_id: u64, milestone_index: u32) -> Result<MilestoneDispute, VerificationError>`

Read a milestone dispute by `(player_id, milestone_index)`. `MilestoneDispute`
has `player_id: u64`, `milestone_index: u32`, `reason: String`,
`disputed_at: u64`, `resolved: bool`, and `upheld: bool`.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `MilestoneNotFound` (no dispute recorded) |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_dispute --player_id 1 --milestone_index 1
```

---

#### `has_dispute(player_id: u64, milestone_index: u32) -> bool`

Boolean convenience check. Returns `true` if a dispute exists for the given
`(player_id, milestone_index)` pair, `false` otherwise (including when no
dispute has ever been submitted or the milestone itself does not exist).

This is a thin read-only wrapper around `get_dispute` — no new storage is
introduced. Mirrors the `is_active_validator` pattern: callers that only need
a yes/no answer (e.g. a frontend showing a "disputed" badge next to a milestone)
avoid handling a `Result`/error path.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- has_dispute --player_id 1 --milestone_index 1
```

---

#### `get_player_dispute_count(player_id: u64) -> u32`

Return the total number of disputes filed for a given `player_id`.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_player_dispute_count --player_id 1
```

---

#### `get_player_disputes(player_id: u64, offset: u32, limit: u32) -> Vec<MilestoneDispute>`

Return a paginated list of all milestone disputes filed for `player_id`.
`offset` is zero-based and `limit` is capped at 50 entries.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_player_disputes --player_id 1 --offset 0 --limit 50
```

---

#### `get_player_disputes_by_status(player_id: u64, resolved: bool, offset: u32, limit: u32) -> Vec<MilestoneDispute>`

Return a paginated list of milestone disputes for `player_id` filtered by resolution status.
If `resolved` is `true`, only resolved disputes are returned. If `resolved` is `false`, only open/unresolved disputes are returned.
`limit` is capped at 50 entries.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- get_player_disputes_by_status --player_id 1 --resolved false --offset 0 --limit 50
```

---

#### `version() -> String`

Return the deployed contract version string (from `Cargo.toml` at build time).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID -- version
```

---

### Events

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `contract_initialized` | event_name, admin (Address) | admin (Address) | Emitted on successful initialization |
| `admin_transfer_proposed` | event_name, old_admin (Address) | new_admin (Address) | Admin replacement proposed |
| `admin_transferred` | event_name, old_admin (Address) | new_admin (Address) | Pending admin accepts control |
| `milestone_approved` | event_name, validator (Address) | player_id (u64), milestone_index (u32), description (String), evidence_hash (String) | Validator confirms a player achievement |
| `validator_registered` | event_name, wallet (Address) | credentials (String) | New validator onboarded |
| `validator_revoked` | event_name, admin (Address) | wallet (Address), reason (String) | Validator deactivated |
| `validator_restored` | event_name, admin (Address) | wallet (Address) | Revoked validator re-activated |
| `validator_transferred` | event_name, admin (Address) | old_wallet (Address), new_wallet (Address) | Validator identity migrated to new wallet |
| `milestone_disputed` | event_name, player_wallet (Address) | player_id (u64), milestone_index (u32), reason (String) | Player disputes a milestone attribution |
| `dispute_resolved` | event_name, admin (Address) | player_id (u64), milestone_index (u32), upheld (bool) | Admin resolves a milestone dispute |
| `progress_contract_updated` | event_name, admin (Address) | progress_contract (Address) | Progress contract re-wired |
| `contract_paused` | event_name, admin (Address) | () | Circuit breaker engaged |
| `contract_unpaused` | event_name, admin (Address) | () | Circuit breaker released |

#### Diagnostic Events (verification)

The following events are emitted for observability when level advancement is skipped or fails. They allow the off-chain indexer to detect silent failures without scanning every transaction receipt for error codes.

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `level_advancement_skipped` | event_name, player_id (u64) | reason (String) | Milestone recorded but level not advanced because player is already at `EliteTier`. `reason` is always `"AlreadyAtMaxLevel"`. Committed to the ledger. |
| `progress_contract_not_set` | event_name, player_id (u64) | `()` | Level advancement skipped because the progress contract address has not been configured. Indicates missing wiring — alert in production. Committed to the ledger. |
| `progress_call_failed` | event_name, player_id (u64) | error_code (u32) | Emitted just before `ProgressCallFailed` is returned. Because that error aborts the entire transaction, this event only appears in the **diagnostic stream** (transaction receipt), not in committed ledger events. `error_code` is the raw error discriminant from `try_advance_level`. |

---

## progress

`ProgressEntry.updated_at` and the `since_timestamp` parameter are Unix
seconds. `ProgressEntry.ledger_sequence` is instead a Soroban ledger sequence
number, not a timestamp. See [Timestamp](GLOSSARY.md#timestamp).

### Functions

---

#### `initialize(admin: Address) -> Result<(), ProgressError>`

One-time contract setup.

| | |
|---|---|
| **Auth** | `admin` must sign |
| **Errors** | `AlreadyInitialized` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- initialize --admin $ADMIN_ADDRESS
```

---

#### `propose_admin(new_admin: Address) -> Result<(), ProgressError>`

Store or replace a pending admin proposal. The current admin retains all
privileges until the proposed address accepts.

| | |
|---|---|
| **Auth** | Current admin must sign |
| **Errors** | `NotInitialized` |
| **Emits** | `admin_transfer_proposed` with `(old_admin, new_admin)` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- propose_admin --new_admin $NEW_ADMIN_ADDRESS
```

---

#### `accept_admin() -> Result<(), ProgressError>`

Finalize the transfer. The stored pending admin must sign, proving control of
the address. Acceptance updates the admin and clears the proposal.

| | |
|---|---|
| **Auth** | Pending admin must sign |
| **Errors** | `NotInitialized` · `PendingAdminNotSet` |
| **Emits** | `admin_transferred` with `(old_admin, new_admin)` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID -- accept_admin
```

---

#### `transfer_admin(new_admin: Address) -> Result<(), ProgressError>`

Deprecated compatibility alias for `propose_admin`. It creates or replaces a
proposal and does not immediately change the admin.

---

#### `reset_player_level(player_id: u64, target_level: ProgressLevel) -> Result<(), ProgressError>`

Reset a player's progress level for dispute resolution or correction.
Existing history is preserved; a new `ProgressEntry` recording the reset is
appended. `milestone_ref` is `0` for admin resets.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` · `ContractPaused` · `Overflow` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- reset_player_level \
  --player_id 1 \
  --target_level '"Unverified"'
```

---

#### `advance_level(caller: Address, player_id: u64, milestone_ref: u32) -> Result<ProgressLevel, ProgressError>`

Advance a player's progress level by one tier. `milestone_ref` links back to
the verification contract's milestone index. Returns the new `ProgressLevel`.

When the verification contract address is configured, only that contract may
invoke this function; otherwise `caller` must sign directly (useful for testing
without a full cross-contract deployment).

| | |
|---|---|
| **Auth** | Verification contract (production) or `caller` directly (test/unconfigured) |
| **Errors** | `NotInitialized` · `ContractPaused` · `AlreadyAtMaxLevel` · `Overflow` · `Unauthorized` |

_Called atomically by `verification.approve_milestone`. Prefer that path in production._

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- advance_level \
  --caller $VALIDATOR_ADDRESS \
  --player_id 1 \
  --milestone_ref 1
```

---

#### `get_level(player_id: u64) -> ProgressLevel`

Return the player's current progress level. Returns `Unverified` for unknown
player IDs (no `PlayerNotFound` error).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- get_level --player_id 1
```

---

#### `get_history_count(player_id: u64) -> u32`

Return the total number of history entries recorded for a player.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- get_history_count --player_id 1
```

---

#### `get_history_entry(player_id: u64, index: u32) -> Result<ProgressEntry, ProgressError>`

Read a specific history entry. Indices start at `1`. Each `ProgressEntry`
includes `updated_at` in Unix seconds and `ledger_sequence: u32`, the Soroban
ledger sequence number at the time of the change (not a timestamp), for
tamper-proof auditability.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `PlayerNotFound` (index out of range) |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- get_history_entry --player_id 1 --index 1
```

---

#### `get_progress_history(player_id: u64) -> Vec<ProgressEntry>`

Return all history entries for a player in chronological order. Internally reads
a single `HistoryVec` persistent storage key regardless of entry count — O(1)
reads instead of the previous O(N) loop. Returns an empty `Vec` for unknown
player IDs.

**Gas trade-off**: the Vec grows with each level change (max 3 entries per player
given the four-tier model). Because the entire Vec is loaded in one read the cost
is proportional to the serialised size of the Vec, not the number of reads.

**Migration note**: existing deployments that only have `HistoryEntry(player_id, i)`
keys (written before this change) will return an empty Vec from this function.
Use `get_history_entry` with individual indices to read pre-migration data, or
run a one-time migration script that calls `advance_level` / `reset_player_level`
to rewrite history into the new Vec key.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- get_progress_history --player_id 1
```

---

#### `pause_contract() -> Result<(), ProgressError>`

Halt all state-changing operations.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID -- pause_contract
```

---

#### `unpause_contract() -> Result<(), ProgressError>`

Resume normal operations after a pause.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID -- unpause_contract
```

---

#### `health() -> ContractHealth`

Return the contract's initialization and pause status.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID -- health
```

---

#### `set_verification_contract(addr: Address) -> Result<(), ProgressError>`

Store the verification contract address so `advance_level` can authenticate cross-contract callers. Without this, only direct `caller` auth is accepted (useful for testing). Admin only.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- set_verification_contract --addr $VERIFICATION_CONTRACT_ID
```

---

#### `set_registration_contract(addr: Address) -> Result<(), ProgressError>`

Store the registration contract address so `advance_level` can sync player levels via cross-contract call. Admin only.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- set_registration_contract --addr $REGISTRATION_CONTRACT_ID
```

---

#### `set_scout_access_contract(addr: Address) -> Result<(), ProgressError>`

Whitelist the scout_access contract as a secondary authorized caller of `advance_level` (for trial-offer Level-3 advances). Admin only.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- set_scout_access_contract --addr $SCOUT_ACCESS_CONTRACT_ID
```

---

#### `upgrade(new_wasm_hash: BytesN<32>) -> Result<(), ProgressError>`

Replace the contract WASM in-place. Persistent storage (admin, history) survives the upgrade. Admin only.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `Unauthorized` · `NotInitialized` |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- upgrade --new_wasm_hash <NEW_WASM_HASH>
```

---

#### `get_progress_history_page(player_id: u64, offset: u32, limit: u32) -> Vec<ProgressEntry>`

Paginated history retrieval. Returns entries from `offset+1` to `offset+limit`. `limit` is capped at 50. Returns an empty `Vec` when `offset` >= total count.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- get_progress_history_page --player_id 1 --offset 0 --limit 10
```

---

#### `get_history_since(player_id: u64, since_timestamp: u64) -> Vec<ProgressEntry>`

Return all of a player's history entries with `updated_at >= since_timestamp`
(Unix seconds). Useful for indexers polling for changes since their last sync
point instead of re-reading the full history.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- get_history_since --player_id 1 --since_timestamp 1700000000
```

---

#### `version() -> String`

Return the deployed contract version string (from `Cargo.toml` at build time).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $PROGRESS_CONTRACT_ID -- version
```

### Events

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `progress_updated` | event_name, updated_by (Address) | player_id (u64), old_level, new_level | Player advances one tier |
| `player_level_reset` | event_name, admin (Address) | player_id (u64), old_level, new_level | Admin resets a player's level |
| `admin_transfer_proposed` | event_name, old_admin (Address) | new_admin (Address) | Admin replacement proposed |
| `admin_transferred` | event_name, old_admin (Address) | new_admin (Address) | Admin rights rotated |
| `contract_paused` | event_name, admin (Address) | () | Circuit breaker engaged |
| `contract_unpaused` | event_name, admin (Address) | () | Circuit breaker released |

---

## scout_access

Handles scout subscriptions, pay-to-contact flows, and trial offer logging.
Fees are collected in XLM (stroops) and held in the contract until admin
withdrawal.

Absolute timestamp fields returned by this contract (`expires_at`,
`subscribed_at`, `contacted_at`, `logged_at`, and `period_start`) are Unix
seconds. `sub_duration_secs` is a duration in seconds, not a Unix timestamp.
See [Timestamp](GLOSSARY.md#timestamp).

### `FeeConfig` Struct

Primary configuration struct controlling all subscription and contact fees.
Passed to `initialize` and `update_fee_config`. All fields must be strictly
greater than zero; either function returns `InvalidInput` otherwise.

| Field | Rust Type | Unit | Valid Range | Typical Example |
|---|---|---|---|---|
| `contact_fee_stroops` | `i128` | stroops (1 XLM = 10 000 000 stroops) | > 0 | `100000` (0.01 XLM) |
| `basic_sub_stroops` | `i128` | stroops | > 0 | `1000000` (0.1 XLM) |
| `pro_sub_stroops` | `i128` | stroops | > 0 | `3000000` (0.3 XLM) |
| `elite_sub_stroops` | `i128` | stroops | > 0 | `7000000` (0.7 XLM) |
| `sub_duration_secs` | `u64` | duration in seconds (not a Unix timestamp) | > 0 | `2592000` (30 days = 30 × 24 × 3600) |

**Validation rules:**
- Every `i128` fee field must be > 0 (zero or negative → `InvalidInput` error code 15).
- `sub_duration_secs` must be > 0 (zero → `InvalidInput`).
- `pro_contact_limit` must be > 0 (zero → `InvalidInput`). This field caps the
  number of unique players a **Pro-tier** scout may contact within a single
  subscription period. Once the limit is reached, `pay_to_contact` returns
  `ProContactLimitReached` (code 20) for that scout until their subscription
  renews. **Elite-tier scouts are exempt** from this limit and may contact any
  number of players regardless of `pro_contact_limit`.
- There is no enforced upper bound on fee fields, but values larger than the XLM supply
  (≈ 500 000 000 XLM = 5 × 10¹⁵ stroops) will cause `Overflow` errors at fee
  settlement time.

> [!NOTE]
> **ContactRecord vs ProContactPeriod — two-tracked quota**
> `ContactRecord` is a **permanent unlock**: created once per `(player_id, scout)` pair
> on successful `pay_to_contact` and never deleted. It gates duplicate-contact checks
> (`AlreadyContacted`).
>
> `ProContactPeriod` (stored under `ProContactCount`) is a **rolling quota counter**:
> it tracks how many *unique* players a **Pro-tier** scout has contacted in the *current
> subscription period* (`period_start == subscription.subscribed_at`). It resets to 0
> automatically when the scout renews/upgrades their subscription. Elite scouts bypass
> this counter entirely.
>
> **Interaction during `pay_to_contact`**: the contract first checks for an existing
> `ContactRecord` (permanent duplicate guard). If none exists and the scout is Pro
> tier, it then checks `ProContactPeriod.count < pro_contact_limit`. On success both
> are written — the permanent `ContactRecord` and the incremented `ProContactPeriod`.

See the [Glossary](GLOSSARY.md#feeconfig) for a plain-language description of each field.

> [!NOTE]
> **Historical Fee Configs & Auditability**
> The `scout_access` contract stores the *current* `FeeConfig` on-chain (retrievable via `get_fee_config`) and a bounded on-chain trail of the **last 5 previous configs** (retrievable via `get_fee_config_history`). The history list is maintained oldest-first and is capped at 5 entries; when the cap is reached the oldest entry is evicted on the next `update_fee_config` call.
>
> This lightweight on-chain trail lets you read the immediately-previous fee configuration without depending on the off-chain indexer, making it suitable for quick audits or on-chain fee-change verification. For a *complete*, unbounded audit trail — including all historical fee rates for verifying that a contact fee or subscription payment matched the rate in effect at that time — replay the `fee_config_updated` event logs via the off-chain indexer's `fee_config_history` table (see [001_initial_schema.sql](migrations/001_initial_schema.sql#L135-L148)).

### Functions

---

#### `initialize(admin: Address, xlm_token: Address, fee_config: FeeConfig) -> Result<(), ScoutAccessError>`

One-time contract setup. Validates that `xlm_token` points at a deployed
token contract by invoking `decimals()` on it, and that all fee fields
are positive with `sub_duration_secs` non-zero. The token probe is
read-only and side-effect-free; it exists so that a wrong `xlm_token`
address (testnet SAC on mainnet, a typo, a plain account, or a
non-token contract) is rejected immediately at deploy time rather than
surfacing as an opaque failure on the first `subscribe()` call.

| | |
|---|---|
| **Auth** | `admin` must sign |
| **Errors** | `AlreadyInitialized` · `InvalidInput` (zero or negative fee field, or `xlm_token` is not a callable token contract) |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- initialize \
  --admin $ADMIN_ADDRESS \
  --xlm_token $XLM_TOKEN_ADDRESS \
  --fee_config '{"contact_fee_stroops":100000,"basic_sub_stroops":1000000,"pro_sub_stroops":3000000,"elite_sub_stroops":7000000,"sub_duration_secs":2592000,"pro_contact_limit":10}'
```

---

#### `propose_admin(new_admin: Address) -> Result<(), ScoutAccessError>`

Store or replace a pending admin proposal. The current admin retains all
privileges until the proposed address accepts.

| | |
|---|---|
| **Auth** | Current admin must sign |
| **Errors** | `NotInitialized` |
| **Emits** | `admin_transfer_proposed` with `(old_admin, new_admin)` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- propose_admin --new_admin $NEW_ADMIN_ADDRESS
```

---

#### `accept_admin() -> Result<(), ScoutAccessError>`

Finalize the transfer. The stored pending admin must sign, proving control of
the address. Acceptance updates the admin and clears the proposal.

| | |
|---|---|
| **Auth** | Pending admin must sign |
| **Errors** | `NotInitialized` · `PendingAdminNotSet` |
| **Emits** | `admin_transferred` with `(old_admin, new_admin)` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID -- accept_admin
```

---

#### `transfer_admin(new_admin: Address) -> Result<(), ScoutAccessError>`

Deprecated compatibility alias for `propose_admin`. It creates or replaces a
proposal and does not immediately change the admin.

---

#### `set_progress_contract(addr: Address) -> Result<(), ScoutAccessError>`

Register the progress contract address so `log_trial_offer` can call
`advance_level` cross-contract (admin only). Unlike
`verification.set_progress_contract`, this has no first-call-only guard —
it can always be re-invoked to re-wire the link.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- set_progress_contract --addr $PROGRESS_CONTRACT_ID
```

---

#### `update_progress_contract(addr: Address) -> Result<(), ScoutAccessError>`

Alias for `set_progress_contract`, provided for naming consistency with
`verification.update_progress_contract` so the same verb can be used to
re-wire the progress contract link across contracts.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- update_progress_contract --addr $NEW_PROGRESS_CONTRACT_ID
```

---

#### `update_fee_config(fee_config: FeeConfig) -> Result<(), ScoutAccessError>`

Adjust subscription and contact fee rates. Same validation rules as
`initialize`.

> [!NOTE]
> **Historical Fee Configs & Auditability**
> Adjusting the fee config emits the `fee_config_updated` event containing both the old and new `FeeConfig` values and also pushes the previous config into the bounded on-chain history (last 5 entries, oldest-first, accessible via `get_fee_config_history`). For a complete unbounded audit trail, replay events into the indexer's `fee_config_history` table (see [001_initial_schema.sql](migrations/001_initial_schema.sql#L135-L148)).

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `Unauthorized` · `InvalidInput` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- update_fee_config \
  --fee_config '{"contact_fee_stroops":200000,"basic_sub_stroops":2000000,"pro_sub_stroops":5000000,"elite_sub_stroops":10000000,"sub_duration_secs":2592000,"pro_contact_limit":20}'
```

---

#### `withdraw_fees(to: Address) -> Result<i128, ScoutAccessError>`

Transfer all accumulated platform fees to the given address. Returns the amount
withdrawn in stroops.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `Unauthorized` · `InsufficientFee` (zero balance) |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- withdraw_fees --to $TREASURY_ADDRESS
```

---

#### `refund_subscription(scout: Address, amount: i128) -> Result<(), ScoutAccessError>`

Emergency admin function to return `amount` XLM (stroops) from the contract
balance to a scout. Use when a scout is accidentally double-charged (e.g. by
the race condition the upgrade timing guard is designed to prevent).

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `Unauthorized` · `InvalidInput` (amount ≤ 0) |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- refund_subscription \
  --scout $SCOUT_ADDRESS \
  --amount 1000000
```

---

#### `subscribe(scout: Address, tier: SubscriptionTier) -> Result<(), ScoutAccessError>`

Purchase a `Basic`, `Pro`, or `Elite` subscription. The XLM fee is transferred
from the scout's wallet to the contract atomically. Downgrades to a cheaper
tier while a subscription is still active are rejected.

> **No-Proration Policy**: Upgrades to a higher tier do **not** provide credit
> for unused time on the previous subscription. The full new-tier fee is charged
> and `expires_at` is reset to `now + sub_duration_secs`. A minimum interval of
> 1 hour between `subscribe` calls from the same scout is enforced to prevent
> race conditions and double-charging.

| | |
|---|---|
| **Auth** | `scout` must sign and pre-approve the XLM transfer |
| **Errors** | `ContractPaused` · `NotInitialized` · `SubscriptionDowngradeNotAllowed` · `UpgradeTooSoon` · `Overflow` |

**Check precedence order** (when multiple error conditions are simultaneously
true, the first matching check in this list wins):

| Priority | Condition checked | Error returned |
|----------|-------------------|---------------|
| 1 | Contract is paused | `ContractPaused` (3) |
| 2 | Contract is not initialized | `NotInitialized` (2) |
| 3 | Scout auth | panic / host auth error |
| 4 | Active subscription exists AND requested tier rank < current tier rank | `SubscriptionDowngradeNotAllowed` (12) |
| 5 | Active subscription exists AND `now < subscribed_at + 3600 s` | `UpgradeTooSoon` (17) |
| 6 | Fee accumulation arithmetic overflows | `Overflow` (10) |
| 7 | `expires_at` calculation overflows | `Overflow` (10) |

> **Design note**: Checks 4 and 5 share the same outer `if` block — only one
> can fire per call. A downgrade attempt is evaluated before the timing guard,
> so a simultaneous downgrade-too-soon scenario returns `SubscriptionDowngradeNotAllowed`.

**Downgrade guard edge cases** (see issue #245 and tests in `scout_access/src/lib.rs`):

| Scenario | Behaviour |
|----------|-----------|
| First-time subscriber (no prior subscription record) | Guard is never reached; any tier may be chosen freely |
| Same-tier re-subscribe while active, after ≥ 1-hour interval | Allowed — `tier_rank(X) < tier_rank(X)` is false, so not a downgrade. `UpgradeTooSoon` still applies within the first hour |
| Same-tier re-subscribe within the first hour | Blocked by `UpgradeTooSoon` (17) — the guard's interval applies to same-tier renewals in addition to upgrades |
| Re-subscribe at exactly `expires_at` timestamp | **Blocked** — the condition is `now <= expires_at`, so the subscription is considered active through its final second. Wait for `now > expires_at` |
| Re-subscribe one second after `expires_at` | Allowed — subscription is expired; any lower tier is permitted |
| Pro (rank 2) → Basic (rank 1) while active | Blocked — `tier_rank(Basic)=1 < tier_rank(Pro)=2` triggers `SubscriptionDowngradeNotAllowed` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- subscribe \
  --scout $SCOUT_ADDRESS \
  --tier '"Elite"'
```

---

#### `pay_to_contact(scout: Address, player_id: u64) -> Result<(), ScoutAccessError>`

Pay a micro-fee to unlock a player's contact details. Scout must have an active
(non-expired) subscription.

**Pro-tier contact limit**: Pro-tier scouts are capped at `pro_contact_limit`
unique player contacts per subscription period (configured in `FeeConfig`).
Once the limit is reached, further `pay_to_contact` calls return
`ProContactLimitReached` (code 20) until the subscription renews. Elite-tier
scouts are **exempt** from this limit.

| | |
|---|---|
| **Auth** | `scout` must sign |
| **Errors** | `ContractPaused` · `NotInitialized` · `ScoutNotSubscribed` · `SubscriptionExpired` · `AlreadyContacted` · `ProContactLimitReached` · `Overflow` |

**Check precedence order** (when multiple error conditions are simultaneously
true, the first matching check in this list wins):

| Priority | Condition checked | Error returned |
|----------|-------------------|---------------|
| 1 | Contract is paused | `ContractPaused` (3) |
| 2 | Contract is not initialized | `NotInitialized` (2) |
| 3 | Scout auth | panic / host auth error |
| 4 | No `Subscription` record exists for the scout | `ScoutNotSubscribed` (6) |
| 5 | `Subscription` record exists but `expires_at < now` | `SubscriptionExpired` (7) |
| 6 | `ContactRecord` already exists for `(player_id, scout)` | `AlreadyContacted` (8) |
| 7 | Scout is Pro tier AND `current_count >= pro_contact_limit` | `ProContactLimitReached` (20) |
| 8 | Fee accumulation arithmetic overflows | `Overflow` (10) |

> **Design note — paused vs unsubscribed (Priority 1 vs 4)**: when the
> contract is paused *and* the scout has no subscription, the caller sees
> `ContractPaused`, not `ScoutNotSubscribed`. A frontend can safely treat
> `ContractPaused` as "service unavailable, try again later" without
> needing to check subscription state. This ordering is intentional and
> consistent with every other state-changing function in this contract.

> **Design note — expired vs already-contacted (Priority 5 vs 6)**: an
> expired subscription takes precedence over a duplicate-contact guard.
> This is the more actionable error for the user ("renew your subscription")
> and prevents leaking whether a contact record exists to an unsubscribed
> caller.

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- pay_to_contact \
  --scout $SCOUT_ADDRESS \
  --player_id 1
```

---

#### `batch_contact_players(scout: Address, player_ids: Vec<u64>) -> Result<u32, ScoutAccessError>`

Contact multiple players in a single transaction. The contact fee is charged
once per new player; already-contacted players are silently skipped (no charge).
The total fee for all new contacts is deducted in a single token transfer.
Returns the count of new contacts recorded.

Scout must have an active (non-expired) subscription.

| | |
|---|---|
| **Auth** | `scout` must sign |
| **Errors** | `ContractPaused` · `NotInitialized` · `ScoutNotSubscribed` · `SubscriptionExpired` · `ContactQuotaExceeded` · `Overflow` |

**Check precedence order** (when multiple error conditions are simultaneously
true, the first matching check in this list wins):

| Priority | Condition checked | Error returned |
|----------|-------------------|---------------|
| 1 | Contract is paused | `ContractPaused` (3) |
| 2 | Contract is not initialized | `NotInitialized` (2) |
| 3 | Scout auth | panic / host auth error |
| 4 | No active subscription (no record or expired) | `ScoutNotSubscribed` (6) or `SubscriptionExpired` (7) |
| 5 | Pro-tier contact quota would be exceeded by the batch | `ContactQuotaExceeded` (18) |
| 6 | `total_fee` multiplication overflows | `Overflow` (10) |

> **Design note — quota check before payment (Priority 5 before fee transfer)**:
> the quota check runs before the XLM transfer. This means no partial charge
> occurs when a batch would exceed the Pro monthly limit — the call fails cleanly
> and the scout can retry with a smaller batch.

> **Design note — `ContactQuotaExceeded` vs `ProContactLimitReached`**: this
> function uses `ContactQuotaExceeded` (18) via the `check_pro_contact_quota_with_count`
> helper, while `pay_to_contact` uses `ProContactLimitReached` (20) via a
> separate inline check. They enforce the same limit but return different error
> codes depending on the call path. Callers should handle both.

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- batch_contact_players \
  --scout $SCOUT_ADDRESS \
  --player_ids '[1,2,3]'
```

---

#### `log_trial_offer(scout: Address, player_id: u64, details_hash: String) -> Result<u32, ScoutAccessError>`

Record a trial offer on-chain. Scout must hold an active Elite subscription.
`details_hash` is an IPFS/Arweave CID of the offer document. Also calls
`progress.advance_level` if the progress contract is registered. Returns the
trial offer index.

| | |
|---|---|
| **Auth** | `scout` must sign (Elite subscription required) |
| **Errors** | `ContractPaused` · `InvalidInput` · `ScoutNotSubscribed` · `SubscriptionExpired` · `Unauthorized` · `TrialOfferRateLimited` · `Overflow` · `ProgressCallFailed` |

**Check precedence order** (when multiple error conditions are simultaneously
true, the first matching check in this list wins):

| Priority | Condition checked | Error returned |
|----------|-------------------|---------------|
| 1 | Contract is paused | `ContractPaused` (3) |
| 2 | Scout auth | panic / host auth error |
| 3 | `details_hash` fails CID validation | `InvalidInput` (15) |
| 4 | No active subscription (no record or expired) | `ScoutNotSubscribed` (6) or `SubscriptionExpired` (7) |
| 5 | Subscription tier is not Elite | `Unauthorized` (4) |
| 6 | No `ContactRecord` exists for `(player_id, scout)` | `Unauthorized` (4) |
| 7 | Rate limit: within 24 h cooldown for `(scout, player_id)` | `TrialOfferRateLimited` (19) |
| 8 | Trial counter increment overflows | `Overflow` (10) |
| 9 | Cross-contract `advance_level` fails for a reason other than `AlreadyAtMaxLevel` | `ProgressCallFailed` (14) |

> ⚠️ **Design note — missing `require_initialized` check**: `log_trial_offer`
> does **not** call `require_initialized`, unlike `subscribe`, `pay_to_contact`,
> and `batch_contact_players`, which all call it immediately after
> `require_not_paused`. This is an asymmetry in the current implementation.
> In practice the function cannot succeed on an uninitialized contract (the
> subscription lookup returns `ScoutNotSubscribed` before any write occurs), but
> callers should not rely on this indirect guard — a dedicated initialized check
> would be safer and consistent. This should be addressed in a follow-up
> contract upgrade. See [Design Discussion §1](#1-log_trial_offer-is-missing-require_initialized).

> **Design note — `InvalidInput` before subscription check (Priority 3 before 4)**:
> `details_hash` is validated before the subscription is looked up. This means
> a scout with an expired subscription who also supplies a malformed CID sees
> `InvalidInput`, not `SubscriptionExpired`. Prefer validating inputs as early
> as possible; this ordering is correct.

> **Design note — both `Unauthorized` codes share priority 5 and 6**: the
> tier check and the previous-contact check both return `Unauthorized` (4)
> but are separate runtime conditions. If a caller has a non-Elite subscription
> *and* has never contacted the player, they will only ever see `Unauthorized`
> from the tier check (priority 5 fires first).

> **Design note — `TrialOfferRateLimited` vs `Unauthorized` ordering
> (Priority 7 after 5–6)**: the rate-limit check occurs after authorization.
> A non-Elite scout cannot trigger `TrialOfferRateLimited`; they will always
> see `Unauthorized` first.

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- log_trial_offer \
  --scout $SCOUT_ADDRESS \
  --player_id 1 \
  --details_hash '"QmTrialOfferDetails"'
```

---

#### `expire_trial_offers(limit: u32) -> Result<u32, ScoutAccessError>`

Admin-only sweep of pending trial offers whose escrow has passed
`expires_at`. For each expired entry it refunds the escrowed XLM to the
originating scout, removes the `TrialEscrow` record, and emits
`trial_offer_expired` — the same cleanup `confirm_trial_offer` performs
reactively when called late, run proactively and in bulk. Returns the
number of escrows actually swept (`0` if none were due).

`limit` bounds how many outstanding escrows are examined in this call,
capped server-side at 20 regardless of the value passed in, so a large
backlog cannot exceed the CPU-instruction budget in a single invocation
(see `ci/cpu-cost-budget.md`). Entries not yet past `expires_at` are left
in place. Call repeatedly (e.g. from a cron/keeper) to drain a backlog
larger than the per-call cap.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` · `Overflow` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- expire_trial_offers --limit 20
```

---

#### `has_contacted(scout: Address, player_id: u64) -> bool`

Return `true` if the scout has previously called `pay_to_contact` for this
player.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- has_contacted \
  --scout $SCOUT_ADDRESS \
  --player_id 1
```

---

#### `get_trial_count(player_id: u64) -> u32`

Return the total number of trial offers logged for a player.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_trial_count --player_id 1
```

---

#### `get_subscription(scout: Address) -> Result<Subscription, ScoutAccessError>`

Read a scout's current subscription record including tier and expiry timestamp.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `ScoutNotSubscribed` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_subscription --scout $SCOUT_ADDRESS
```

---

#### `get_fee_config() -> FeeConfig`

Return the current fee configuration.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID -- get_fee_config
```

---

#### `get_fee_config_history() -> Vec<FeeConfigHistoryEntry>`

Return the bounded on-chain history of the last (up to 5) `FeeConfig` values, **oldest-first**.

Each `FeeConfigHistoryEntry` contains:
- `config: FeeConfig` — the fee configuration that was active *before* a particular `update_fee_config` call.
- `updated_at: u64` — the Unix-seconds ledger timestamp when that change was made.

The *current* config is not included — retrieve it with `get_fee_config`. The history grows by
one entry per `update_fee_config` call and is capped at 5 entries; when the cap is reached the
oldest entry is evicted. This provides a lightweight middle-ground between the indexer-only
design (full history via `fee_config_updated` events) and an unbounded on-chain ring-buffer,
keeping the immediately-previous configs readable on-chain without additional indexer dependency.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID -- get_fee_config_history
```

---

#### `get_accumulated_fees() -> i128`

Return total platform fees pending admin withdrawal (in stroops).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID -- get_accumulated_fees
```

---

#### `get_trial_offer(player_id: u64, index: u32) -> Result<TrialOffer, ScoutAccessError>`

Read a specific trial offer. Indices start at `1`.

| | |
|---|---|
| **Auth** | None |
| **Errors** | `TrialOfferNotFound` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_trial_offer --player_id 1 --index 1
```

---

#### `pause_contract() -> Result<(), ScoutAccessError>`

Halt all state-changing operations.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID -- pause_contract
```

---

#### `unpause_contract() -> Result<(), ScoutAccessError>`

Resume normal operations after a pause.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `NotInitialized` · `Unauthorized` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID -- unpause_contract
```

---

#### `health() -> ContractHealth`

Return the contract's initialization and pause status.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID -- health
```

---

#### `upgrade(new_wasm_hash: BytesN<32>) -> Result<(), ScoutAccessError>`

Replace the contract WASM in-place. Persistent storage (admin, subscriptions, trial offers) survives the upgrade. Admin only.

| | |
|---|---|
| **Auth** | Admin must sign |
| **Errors** | `Unauthorized` · `NotInitialized` |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- upgrade --new_wasm_hash <NEW_WASM_HASH>
```

---

#### `get_scout_contacts(scout: Address) -> Vec<u64>`

Return all player IDs contacted by a scout as an O(1) index lookup (backed by `ScoutContacts` persistent storage key).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_scout_contacts --scout $SCOUT_ADDRESS
```

---

#### `get_all_trial_offers(player_id: u64) -> Vec<TrialOffer>`

Return all trial offers for a player in a single call. Bounded at 20 to prevent gas exhaustion. Returns an empty `Vec` when no offers exist.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_all_trial_offers --player_id 1
```

---

#### `get_subscribers_by_tier(tier: SubscriptionTier) -> Vec<Address>`

Return all scout addresses currently subscribed at `tier` (an O(1) index
lookup backed by the `TierSubscribers` persistent storage key). Includes
expired subscriptions that have not yet been superseded by a renewal or
downgrade.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_subscribers_by_tier --tier '"Elite"'
```

---

#### `get_contact_record(scout: Address, player_id: u64) -> Option<ContactRecord>`

Return the full `ContactRecord` for a `(scout, player_id)` pair, or `None`
if the scout has never contacted this player.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_contact_record --scout $SCOUT_ADDRESS --player_id 1
```

---

#### `get_player_contacts(player_id: u64) -> Vec<Address>`

Return all scout addresses that have contacted a player, as an O(1) index
lookup (backed by the `PlayerContacts` persistent storage key).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_player_contacts --player_id 1
```

---

#### `get_player_trial_offers(player_id: u64) -> Vec<TrialOffer>`

Return every trial offer logged for a player, reading the full range from
the player's `TrialCounter`. Unlike `get_all_trial_offers`, this is not
capped at 20 entries.

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_player_trial_offers --player_id 1
```

---

#### `get_scout_trial_offers(scout: Address) -> Vec<(u64, u32)>`

Return every `(player_id, trial_offer_index)` pair a scout has logged, as
an O(1) index lookup (backed by the `ScoutTrialOffers` persistent storage
key).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- get_scout_trial_offers --scout $SCOUT_ADDRESS
```

---

#### `version() -> String`

Return the deployed contract version string (from `Cargo.toml` at build time).

| | |
|---|---|
| **Auth** | None |
| **Errors** | None |

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID -- version
```

### Events

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `contract_initialized` | event_name, admin (Address) | admin (Address) | Emitted on successful initialization |
| `scout_subscribed` | event_name, scout (Address) | (tier: SubscriptionTier, fee_paid: i128) | Scout purchases a subscription |
| `player_contacted` | event_name, scout (Address) | (player_id: u64, fee_paid: i128) | Scout unlocks player contact details |
| `trial_offer_logged` | event_name, scout (Address) | player_id (u64) | Elite scout records a trial offer |
| `fees_withdrawn` | event_name, admin (Address) | (to: Address, amount: i128, timestamp: u64) | Admin withdraws accumulated fees |
| `subscription_refunded` | event_name, scout (Address) | amount (i128) | Admin issues emergency refund to a scout |
| `admin_transfer_proposed` | event_name, old_admin (Address) | new_admin (Address) | Admin replacement proposed |
| `admin_transferred` | event_name, old_admin (Address) | new_admin (Address) | Admin rights rotated |
| `contract_paused` | event_name, admin (Address) | () | Circuit breaker engaged |
| `contract_unpaused` | event_name, admin (Address) | () | Circuit breaker released |
| `progress_contract_updated` | event_name, admin (Address) | progress_contract (Address) | Progress contract re-wired |
| `fee_config_updated` | event_name, admin (Address) | (old_config: FeeConfig, new_config: FeeConfig) | Fee configuration changed |
| `subscription_created` | event_name, scout (Address) | (tier, subscribed_at, expires_at) | New subscription purchased (first-ever for this scout) |
| `subscription_renewed` | event_name, scout (Address) | (tier, subscribed_at, expires_at) | Existing subscription renewed or upgraded |

#### Diagnostic Events (scout_access)

The following events are emitted from `confirm_trial_offer` when level advancement via the progress contract is skipped or fails.

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `progress_contract_not_set` | event_name, player_id (u64) | `()` | `confirm_trial_offer` could not advance the player's level because the progress contract address has not been wired. Emitted before returning `InvalidInput`. Indicates missing wiring — alert in production. Committed to the ledger. |
| `progress_call_failed` | event_name, player_id (u64) | error_code (u32) | Emitted just before `ProgressCallFailed` is returned from `confirm_trial_offer`. Because that error aborts the entire transaction, this event only appears in the **diagnostic stream** (transaction receipt), not in committed ledger events. `error_code` is the raw error discriminant from `try_advance_level`. |

---

## Shared Types

### `ProgressLevel`

Four-tier progress level used by all contracts. It is the core player ranking type
referenced throughout registration, verification, progress, and scout_access.

#### Variant table

| Ordinal | Variant | Semantic meaning |
|---------|---------|-----------------|
| 0 | `Unverified` | Profile created on-chain; no identity or performance verification has occurred yet. Default state for all newly registered players. |
| 1 | `VerifiedIdentity` | Identity confirmed by an approved academy or KYC validator. Player is discoverable by scouts with a Basic subscription or higher. |
| 2 | `PerformanceMilestones` | Performance statistics verified by an approved third-party validator. Player is discoverable by scouts with a Pro subscription or higher. |
| 3 | `EliteTier` | Scout feedback or a trial offer has been logged by an Elite-tier scout. Player is discoverable by scouts with an Elite subscription only. |

#### Subscription tier access mapping

| ProgressLevel | Minimum subscription tier to view |
|---------------|----------------------------------|
| `Unverified` (0) | None — public profile metadata only (no contact) |
| `VerifiedIdentity` (1) | Basic |
| `PerformanceMilestones` (2) | Pro |
| `EliteTier` (3) | Elite |

Scouts without a sufficient tier can still see that a player exists but cannot view
full profile details or initiate contact. Contact actions are separately gated by
`scout_access.contact_player`.

#### Valid transitions

Levels advance sequentially: 0 → 1 → 2 → 3. No skipping or reversing is permitted
except via the admin function `progress.reset_player_level`.

Level promotion is triggered by `verification.approve_milestone`, which cross-calls
[`progress.advance_level`](#advance_level-caller-address-player_id-u64-milestone_ref-u32---resultprogresslevel-progresserror).
The new level is also reflected in `registration` queries, including
[`registration.filter_players`](#filter_players-region-string-position-string-min_level-progresslevel---resultvecplayerprofile-scoutchainerror),
which accepts a `min_level` argument to restrict results to players at or above a
given tier.

### `ContractHealth`

```rust
pub struct ContractHealth {
    pub initialized: bool,
    pub paused: bool,
}
```

### `PlayerVitals`

```rust
pub struct PlayerVitals {
    pub age: u32,
    pub position: String,  // max 64 bytes
    pub region: String,    // max 64 bytes
    pub nationality: String, // max 64 bytes
}
```

### `PlayerProfile`

```rust
pub struct PlayerProfile {
    pub player_id: u64,
    pub wallet: Address,
    pub vitals: PlayerVitals,
    pub ipfs_hashes: Vec<String>, // 1–10 entries
    pub level: ProgressLevel,
    pub registered_at: u64, // Unix seconds
    pub updated_at: u64,    // Unix seconds
}
```

### `ScoutProfile`

```rust
pub struct ScoutProfile {
    pub scout_id: u64,
    pub wallet: Address,
    pub region: String,   // max 128 bytes
    pub verified: bool,
    pub registered_at: u64, // Unix seconds
}
```

### `Validator`

```rust
pub struct Validator {
    pub wallet: Address,
    pub credentials: String, // max 256 bytes
    pub registered_at: u64, // Unix seconds
    pub active: bool,
}
```

### `ValidatorStatus`

```rust
pub enum ValidatorStatus {
    NotRegistered,
    Active,
    Revoked,
}
```

### `Milestone`

```rust
pub struct Milestone {
    pub player_id: u64,
    pub validator: Address,
    pub description: String,
    pub evidence_hash: String,  // IPFS Qm… or Arweave bafy…, 2–128 bytes
    pub approved_at: u64,       // Unix seconds
    pub ledger_sequence: u32,   // Soroban ledger sequence number (not a timestamp)
}
```

### `MilestoneDispute`

```rust
pub struct MilestoneDispute {
    pub player_id: u64,
    pub milestone_index: u32,
    pub reason: String,
    pub disputed_at: u64,       // Unix seconds
    pub resolved: bool,         // false until admin resolves the dispute
    pub upheld: bool,           // admin outcome; meaningful once resolved is true
}
```

### `ProgressEntry`

```rust
pub struct ProgressEntry {
    pub player_id: u64,
    pub old_level: ProgressLevel,
    pub new_level: ProgressLevel,
    pub updated_by: Address,
    pub updated_at: u64,        // Unix seconds
    pub milestone_ref: u32,     // links to verification contract index
    pub ledger_sequence: u32,   // Soroban ledger sequence number (not a timestamp)
}
```

### `SubscriptionTier`

```rust
pub enum SubscriptionTier {
    Basic,  // browse Level 1+ players
    Pro,    // browse all levels + up to 10 contacts/month
    Elite,  // unlimited contacts + trial offer logging
}
```

### `Subscription`

```rust
pub struct Subscription {
    pub scout: Address,
    pub tier: SubscriptionTier,
    pub expires_at: u64,        // Unix seconds
    pub subscribed_at: u64,     // Unix seconds
}
```

### `ContactRecord`

```rust
pub struct ContactRecord {
    pub player_id: u64,
    pub scout: Address,
    pub contacted_at: u64,      // Unix seconds
}
```

### `FeeConfig`

```rust
pub struct FeeConfig {
    pub contact_fee_stroops: i128,   // must be > 0
    pub basic_sub_stroops: i128,     // must be > 0
    pub pro_sub_stroops: i128,       // must be > 0
    pub elite_sub_stroops: i128,     // must be > 0
    pub sub_duration_secs: u64,      // duration in seconds, must be > 0 (not a Unix timestamp)
}
```

> [!NOTE]
> **Historical Fee Configs & Auditability**
> The `scout_access` contract only stores the *current* `FeeConfig` on-chain. Historical fee configurations are event-log-only and must be reconstructed off-chain by replaying `fee_config_updated` events into the indexer's `fee_config_history` table (defined in [001_initial_schema.sql](file:///c:/Users/USER/scout-off-contracts/migrations/001_initial_schema.sql#L135-L148)).


### `ProContactPeriod`

```rust
pub struct ProContactPeriod {
    pub period_start: u64,      // Unix seconds
    pub count: u32,
}
```

### `TrialOffer`

```rust
pub struct TrialOffer {
    pub player_id: u64,
    pub scout: Address,
    pub details_hash: String, // IPFS/Arweave CID
    pub logged_at: u64,         // Unix seconds
}
```

---

## Error Codes

### `ScoutChainError` (registration contract)

| Code | Variant | Common Cause |
|------|---------|--------------|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `NotInitialized` | Operation before `initialize` |
| 3 | `PlayerNotFound` | Invalid `player_id` |
| 4 | `ValidatorNotAuthorized` | Unregistered account approving milestone |
| 5 | `InvalidProgressTransition` | Skipping or reversing a level |
| 6 | `ScoutNotSubscribed` | Scout has no subscription |
| 7 | `InsufficientFee` | Underpaying contact fee |
| 8 | `AlreadyRegistered` | Wallet already has a profile for this role |
| 9 | `ContractPaused` | Circuit breaker is active |
| 10 | `Unauthorized` | Wrong account for a privileged operation |
| 11 | `Overflow` | Counter or fee arithmetic overflowed |
| 12 | `ScoutNotFound` | Invalid `scout_id` |
| 13 | `InvalidInput` | Field too long, bad hash count, or empty value |
| 14 | `PendingAdminNotSet` | `accept_admin` called without a pending proposal |

### `VerificationError` (verification contract)

| Code | Variant | Common Cause |
|------|---------|--------------|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `NotInitialized` | Operation before `initialize` |
| 3 | `ContractPaused` | Circuit breaker is active |
| 4 | `Unauthorized` | Wrong account for a privileged operation |
| 5 | `ValidatorNotFound` | Wallet not in validator registry |
| 6 | `ValidatorInactive` | Validator has been revoked |
| 7 | `ValidatorAlreadyRegistered` | Wallet already registered as validator |
| 8 | `PlayerNotFound` | Invalid `player_id` |
| 9 | `InvalidInput` | Bad evidence hash or credentials too long |
| 10 | `ReasonTooLong` | Revocation reason exceeds 128 bytes |
| 11 | `AlreadyConfigured` | `set_progress_contract` called twice |
| 12 | `ProgressCallFailed` | Cross-contract `advance_level` failed |
| 13 | `Overflow` | Milestone counter overflowed |
| 14 | `MilestoneNotFound` | Index out of range |
| 15 | `ValidatorCapReached` | 100-validator limit reached; contract upgrade required to raise the cap |
| 16 | `DuplicateEvidence` | Evidence hash has already been used in a prior `approve_milestone` call |
| 17 | `MilestoneLimitExceeded` | Validator has already approved 5 milestones for this player |
| 18 | `DisputeAlreadyResolved` | Dispute was already resolved and cannot be resolved again |
| 19 | `PendingAdminNotSet` | `accept_admin` called without a pending proposal |

### `ProgressError` (progress contract)

| Code | Variant | Common Cause |
|------|---------|--------------|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `NotInitialized` | Operation before `initialize` |
| 3 | `ContractPaused` | Circuit breaker is active |
| 4 | `Unauthorized` | Wrong account for a privileged operation |
| 5 | `InvalidProgressTransition` | Level skip or reversal attempted |
| 6 | `AlreadyAtMaxLevel` | Player is already at `EliteTier` |
| 7 | `PlayerNotFound` | History index out of range |
| 8 | `Overflow` | History counter overflowed |
| 9 | `RegistrationCallFailed` | Cross-contract call to registration contract failed when syncing player level |
| 10 | `PendingAdminNotSet` | `accept_admin` called without a pending proposal |

### `ScoutAccessError` (scout_access contract)

| Code | Variant | Common Cause |
|------|---------|--------------|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `NotInitialized` | Operation before `initialize` |
| 3 | `ContractPaused` | Circuit breaker is active |
| 4 | `Unauthorized` | Wrong account or non-Elite tier for trial offer |
| 5 | `InsufficientFee` | Scout underpaid a subscription or contact fee |
| 6 | `ScoutNotSubscribed` | No subscription record found |
| 7 | `SubscriptionExpired` | Subscription past `expires_at` |
| 8 | `AlreadyContacted` | Duplicate `pay_to_contact` for same player |
| 9 | `InvalidTier` | Unknown subscription tier |
| 10 | `Overflow` | Fee accumulation arithmetic overflowed |
| 11 | `TrialOfferNotFound` | Index out of range |
| 12 | `SubscriptionDowngradeNotAllowed` | Downgrade attempted while subscription active |
| 14 | `ProgressCallFailed` | Cross-contract `advance_level` failed |
| 15 | `InvalidInput` | Zero or negative fee field in `FeeConfig` |
| 16 | `NoFeesToWithdraw` | No accumulated fees available to withdraw |
| 17 | `UpgradeTooSoon` | Subscribe called before minimum interval elapsed |
| 18 | `ContactQuotaExceeded` | Scout has hit the platform-wide contact quota for the current period (applies to all tiers; enforced by an admin-configurable platform cap, distinct from the per-Pro-scout `pro_contact_limit`) |
| 19 | `TrialOfferRateLimited` | Elite scout sent a trial offer to the same player within the cooldown window — the offer was already logged; retry after the cooldown expires |
| 20 | `ProContactLimitReached` | Pro-tier scout has reached the `pro_contact_limit` contacts for the current subscription period (Elite scouts are exempt from this limit) |
| 21 | `PendingAdminNotSet` | `accept_admin` called before an admin transfer was proposed via `propose_admin` |
| 22 | `TrialOfferAlreadyConfirmed` | `confirm_trial_offer` called twice for the same trial offer |
| 23 | `TrialOfferExpired` | `confirm_trial_offer` called after the offer's confirmation window elapsed |

---

## Events

All events follow the unified `(Symbol, actor)` topic schema introduced in #246. Soroban event indexers can filter any event by actor address using the second topic element.

**Standard schema**: `topics: (event_name: Symbol, actor: Address)` · `data: (entity_id, ...other_fields)`

### registration

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `player_registered` | event_name, wallet (Address) | player_id (u64) | New player profile created |
| `scout_registered` | event_name, wallet (Address) | scout_id (u64) | New scout profile created |
| `profile_updated` | event_name, wallet (Address) | player_id (u64) | Player updates IPFS content hashes |
| `player_deregistered` | event_name, admin (Address) | player_id (u64) | Admin removes a player profile |
| `player_deactivated` | event_name, admin (Address) | player_id (u64) | Admin soft-hides a player from filter results |
| `player_reactivated` | event_name, admin (Address) | player_id (u64) | Admin restores a soft-hidden player to filter results |
| `scout_verified` | event_name, wallet (Address) | scout_id (u64) | Admin verifies a scout |
| `player_level_synced` | event_name, progress_contract (Address) | player_id (u64) | Progress contract syncs a player's level |
| `admin_transfer_proposed` | event_name, old_admin (Address) | new_admin (Address) | Current admin proposes a replacement |
| `admin_transferred` | event_name, old_admin (Address) | new_admin (Address) | Pending admin accepts control |

### verification

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `contract_initialized` | event_name, admin (Address) | admin (Address) | Contract initialized |
| `admin_transfer_proposed` | event_name, old_admin (Address) | new_admin (Address) | Current admin proposes a replacement |
| `admin_transferred` | event_name, old_admin (Address) | new_admin (Address) | Pending admin accepts control |
| `milestone_approved` | event_name, validator (Address) | player_id (u64), milestone_index (u32), description (String), evidence_hash (String) | Validator confirms a player achievement |
| `validator_registered` | event_name, wallet (Address) | credentials (String) | New validator onboarded |
| `validator_revoked` | event_name, admin (Address) | wallet (Address), reason (String) | Validator deactivated |
| `validator_restored` | event_name, admin (Address) | wallet (Address) | Revoked validator re-activated |
| `validator_transferred` | event_name, admin (Address) | old_wallet (Address), new_wallet (Address) | Validator identity migrated to new wallet |
| `milestone_disputed` | event_name, player_wallet (Address) | player_id (u64), milestone_index (u32), reason (String) | Player disputes a milestone attribution |
| `dispute_resolved` | event_name, admin (Address) | player_id (u64), milestone_index (u32), upheld (bool) | Admin resolves a milestone dispute |
| `progress_contract_updated` | event_name, admin (Address) | progress_contract (Address) | Progress contract address re-wired |
| `contract_paused` | event_name, admin (Address) | () | Circuit breaker engaged |
| `contract_unpaused` | event_name, admin (Address) | () | Circuit breaker released |

### progress

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `progress_updated` | event_name, updated_by (Address) | player_id (u64), old_level, new_level | Player advances one level |
| `player_level_reset` | event_name, admin (Address) | player_id (u64), old_level, new_level | Admin resets a player's level |
| `admin_transfer_proposed` | event_name, old_admin (Address) | new_admin (Address) | Current admin proposes a replacement |
| `admin_transferred` | event_name, old_admin (Address) | new_admin (Address) | Pending admin accepts control |
| `contract_paused` | event_name, admin (Address) | () | Circuit breaker engaged |
| `contract_unpaused` | event_name, admin (Address) | () | Circuit breaker released |

### scout_access

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `contract_initialized` | event_name, admin (Address) | admin (Address) | Contract initialized |
| `scout_subscribed` | event_name, scout (Address) | tier (SubscriptionTier), fee_paid (i128) | Scout purchases a subscription |
| `subscription_created` | event_name, scout (Address) | tier, subscribed_at (u64), expires_at (u64) | First-ever subscription for this scout |
| `subscription_renewed` | event_name, scout (Address) | tier, subscribed_at (u64), expires_at (u64) | Existing subscription renewed or upgraded |
| `player_contacted` | event_name, scout (Address) | player_id (u64), fee_paid (i128) | Scout unlocks player contact details |
| `trial_offer_logged` | event_name, scout (Address) | player_id (u64) | Elite scout records a trial offer |
| `fees_withdrawn` | event_name, admin (Address) | to (Address), amount (i128), timestamp (u64) | Admin withdraws accumulated fees |
| `subscription_refunded` | event_name, scout (Address) | amount (i128) | Admin issues emergency refund to a scout |
| `fee_config_updated` | event_name, admin (Address) | old_config (FeeConfig), new_config (FeeConfig) | Fee configuration changed |
| `progress_contract_updated` | event_name, admin (Address) | progress_contract (Address) | Progress contract re-wired |
| `admin_transfer_proposed` | event_name, old_admin (Address) | new_admin (Address) | Current admin proposes a replacement |
| `admin_transferred` | event_name, old_admin (Address) | new_admin (Address) | Pending admin accepts control |
| `contract_paused` | event_name, admin (Address) | () | Circuit breaker engaged |
| `contract_unpaused` | event_name, admin (Address) | () | Circuit breaker released |

---

## Design Discussion: Check-Ordering Follow-ups

This section collects ordering decisions that were identified during the
check-precedence audit and flagged as candidates for review in a future
contract upgrade. None of these represent bugs in the current release —
all of them have documented, tested behavior — but some may produce a less
helpful error than a different ordering would. Each item describes the
current behavior, why it may be suboptimal, and the recommended change.

---

### 1. `log_trial_offer` is missing `require_initialized`

**Current behavior**: `log_trial_offer` does not call `require_initialized`,
unlike every other state-changing function in this contract (`subscribe`,
`pay_to_contact`, and `batch_contact_players` all call it immediately after
`require_not_paused`).

**Why this matters**: On an uninitialized contract, `log_trial_offer` does not
return `NotInitialized`. Instead it falls through to the subscription lookup,
which returns `ScoutNotSubscribed` because no storage has been written. This
means an uninitialized contract appears to a caller as if the scout simply
has no subscription — an indirect, misleading error rather than the definitive
"contract not set up" signal.

**When it can surface**: Only on a freshly deployed contract that has never had
`initialize` called. In production the initialize-then-use deployment flow
makes this unlikely, but a mis-wired deployment or a test environment that
calls `log_trial_offer` before `initialize` would observe `ScoutNotSubscribed`
instead of `NotInitialized`.

**Recommended fix**: Add `Self::require_initialized(&env)?;` immediately after
`Self::require_not_paused(&env)?;` in `log_trial_offer`, matching the ordering
of the other three state-changing functions. This is a one-line change, is
backward-compatible (it makes an already-failing path fail with a more specific
error), and requires no storage or API changes.

```rust
// Proposed change in log_trial_offer (contracts/scout_access/src/lib.rs):
Self::bump_instance_ttl(&env);
Self::require_not_paused(&env)?;
Self::require_initialized(&env)?;   // ← add this line
scout.require_auth();
```

**Risk**: None. On an initialized contract `require_initialized` always
succeeds, so existing callers are unaffected.

---

### 2. `pay_to_contact`: `AlreadyContacted` checked before `ProContactLimitReached` (Priority 6 before 7)

**Current behavior**: The duplicate-contact guard (`AlreadyContacted`) runs
before the Pro monthly quota check (`ProContactLimitReached`). A scout who
is simultaneously at their quota limit *and* has already contacted the same
player sees `AlreadyContacted`.

**Why this may be suboptimal**: `AlreadyContacted` (code 8) is the correct
terminal error for a genuine duplicate contact attempt, so the ordering is
correct for the pure-duplicate case. However, the quota check at Priority 7
fires *only* for new contacts — if a scout at quota tries to contact a new
player they will correctly see `ProContactLimitReached`. The current ordering
is therefore only relevant when both the quota and a duplicate exist for the
same `(scout, player_id)` pair. In that case `AlreadyContacted` is the more
actionable response ("you already unlocked this player") and the quota is
irrelevant. The current ordering is defensible.

**Conclusion**: No change recommended. The ordering is correct and the
"worse" scenario (quota masking duplicate) does not arise in practice because
the quota check only runs for *new* contacts.

---

### 3. `batch_contact_players` vs `pay_to_contact`: different error codes for the same quota limit

**Current behavior**: `batch_contact_players` returns `ContactQuotaExceeded`
(18) when the Pro monthly limit would be exceeded, while `pay_to_contact`
returns `ProContactLimitReached` (20) for the same underlying limit. Both
enforce `pro_contact_limit` from `FeeConfig` but via different helper
functions.

**Why this matters for callers**: A frontend must handle two different error
codes to display the same user-facing message ("You have reached your monthly
contact limit, please upgrade to Elite or wait for your subscription to
renew"). This is an accidental inconsistency introduced when `batch_contact_players`
was added.

**Recommended fix**: Unify on one error code. The preferred candidate is
`ProContactLimitReached` (20) because it is the more descriptive name and was
introduced specifically for this error class. `ContactQuotaExceeded` (18) can
be deprecated and its slot reserved (see the code-13 reservation pattern
already in use in `errors.rs`). This requires a contract upgrade and a
coordinated frontend change.

**Impact**: Any caller or frontend that currently checks for
`ContactQuotaExceeded` (18) on `batch_contact_players` responses would need to
be updated after the upgrade.

---

### 4. `subscribe`: UpgradeTooSoon fires even for a same-tier renewal

**Current behavior**: the minimum 1-hour interval between `subscribe` calls
(the `UpgradeTooSoon` guard) applies to any call while the subscription is
active, including a renewal at exactly the same tier. A scout attempting to
renew their Pro subscription 30 minutes after purchasing it sees `UpgradeTooSoon`.

**Why this may be suboptimal**: The guard was introduced to prevent the
race-condition / double-charge scenario on rapid upgrades. A same-tier renewal
carries no race-condition risk because the tier does not change and the fee
is deterministic. Applying the interval guard to same-tier renewals is a
conservative over-application that can confuse users ("I'm just renewing,
why is it saying too soon?").

**Recommended fix**: Only apply the `UpgradeTooSoon` guard when the requested
tier is a strict upgrade (i.e., `tier_rank(&tier) > tier_rank(&existing.tier)`).
Same-tier renewals while active should only be rate-limited by the expiry
logic, not the upgrade interval. This is a small conditional change within the
existing `if now <= existing.expires_at` block.

**Risk**: Low. Removing the interval guard for same-tier renewals means two
identical-tier subscriptions *could* be purchased in rapid succession (paying
double). However, this is self-penalizing (the scout pays twice for no
benefit) and the new subscription simply overwrites the old one. The
`refund_subscription` admin function already handles the accidental-double-charge
recovery path.
