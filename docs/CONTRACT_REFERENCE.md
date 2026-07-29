# Contract Reference

Quick reference for all four ScoutChain Soroban contracts.

---

## registration

Handles player and scout on-chain identity.

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin)` | admin | One-time setup |
| `register_player(wallet, vitals, ipfs_hashes)` | wallet | Create player profile at Level 0 |
| `update_profile(player_id, ipfs_hashes)` | player wallet | Update IPFS content hashes |
| `register_scout(wallet, region)` | wallet | Create scout profile |
| `get_player(player_id)` | — | Read player profile |
| `get_player_by_wallet(wallet)` | — | Lookup player by wallet |
| `get_scout(scout_id)` | — | Read scout profile |
| `get_player_count()` | — | Total registered players |
| `get_scout_count()` | — | Total registered scouts |
| `pause_contract()` / `unpause_contract()` | admin | Circuit breaker |
| `health()` | — | Returns true if initialized |

### Dual-Role Wallet Policy

A single wallet address **may register as both a player and a scout**. This is intentional and allowed. A wallet can hold both roles simultaneously without restriction. Duplicate prevention is enforced per role (a wallet cannot register twice as a player, and cannot register twice as a scout), but cross-role registration is permitted.

---

## verification

Manages the trusted validator registry and milestone approvals.

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin)` | admin | One-time setup, including default jury configuration |
| `set_progress_contract(progress_contract)` | admin | Wire cross-contract link |
| `set_jury_config(impact_threshold, quorum, voting_window_secs)` | admin | Configure future jury disputes |
| `register_validator(wallet, credentials)` | admin | Add trusted validator |
| `revoke_validator(wallet)` | admin | Deactivate validator |
| `approve_milestone(validator_wallet, player_id, description, evidence_hash)` | validator | Record milestone (with ledger_sequence for audit) + cross-call progress.advance_level |
| `dispute_milestone(filed_by, player_id, milestone_index, reason, impact_score)` | filer | File a low-impact or jury-required dispute |
| `resolve_dispute(player_id, milestone_index, upheld)` | admin | Finalize a low-impact dispute |
| `cast_dispute_vote(validator_wallet, player_id, milestone_index, upheld)` | active, non-conflicted validator | Cast one jury vote |
| `tally_dispute(player_id, milestone_index)` | — | Finalize a decisive quorum or expired jury vote |
| `get_milestone(player_id, index)` | — | Read a specific milestone |
| `get_milestone_count(player_id)` | — | Total milestones for a player |
| `get_jury_config()` | — | Read jury threshold, quorum, and voting window |
| `get_dispute(player_id, milestone_index)` | — | Read a dispute and its tally |
| `get_dispute_vote(player_id, milestone_index, validator_wallet)` | — | Read one validator vote |
| `get_dispute_votes(player_id, milestone_index)` | — | Read upheld and rejected vote totals |
| `get_validator(wallet)` | — | Read validator record |
| `is_active_validator(wallet)` | — | Boolean check |
| `pause_contract()` / `unpause_contract()` | admin | Circuit breaker |
| `health()` | — | Returns true if initialized |

### Events

| Event | Topics | Data | Description |
|-------|--------|------|-------------|
| `milestone_approved` | event_name, validator_address, milestone_index (u32) | player_id (u64), description (String), evidence_hash (String) | Emitted when a validator approves a player milestone with full milestone details |
| `validator_registered` | event_name | validator_address | Emitted when a new validator is registered |
| `validator_revoked` | event_name | validator_address | Emitted when a validator is deactivated |
| `milestone_disputed` | event_name, player_id, milestone_index | filer_address, jury_required | Emitted when a dispute is filed |
| `dispute_vote_cast` | event_name, player_id, milestone_index | validator_address, upheld | Emitted for each jury vote |
| `dispute_resolved` | event_name, player_id, milestone_index | upheld | Emitted after an admin resolution |
| `dispute_tallied` | event_name, player_id, milestone_index | upheld, votes_for, votes_against | Emitted after jury finalization |

---

## progress

Maintains the tamper-proof four-tier level state machine.

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin)` | admin | One-time setup |
| `advance_level(caller, player_id, milestone_ref)` | caller (validator or scout) | Move player up one level |
| `get_level(player_id)` | — | Current progress level |
| `get_history_count(player_id)` | — | Number of level changes |
| `get_history_entry(player_id, index)` | — | Specific history entry |
| `pause_contract()` / `unpause_contract()` | admin | Circuit breaker |
| `health()` | — | Returns true if initialized |

---

## scout_access

Handles scout subscriptions, pay-to-contact, and trial offer logging.

| Function | Auth | Description |
|----------|------|-------------|
| `initialize(admin, xlm_token, fee_config)` | admin | One-time setup |
| `update_fee_config(fee_config)` | admin | Adjust fee rates |
| `withdraw_fees(to)` | admin | Collect platform revenue |
| `subscribe(scout, tier)` | scout | Purchase Basic/Pro/Elite subscription |
| `pay_to_contact(scout, player_id)` | scout | Pay micro-fee to unlock player contact |
| `log_trial_offer(scout, player_id, details_hash)` | scout (Elite only) | Record trial offer on-chain |
| `get_subscription(scout)` | — | Read subscription record |
| `get_fee_config()` | — | Current fee configuration |
| `get_accumulated_fees()` | — | Platform fees pending withdrawal |
| `has_contacted(scout, player_id)` | — | Boolean contact check |
| `get_trial_offer(player_id, index)` | — | Read a trial offer |
| `get_trial_count(player_id)` | — | Total trial offers for a player |
| `pause_contract()` / `unpause_contract()` | admin | Circuit breaker |
| `health()` | — | Returns true if initialized |

---

## Progress Levels

| Integer | Enum | Trigger |
|---------|------|---------|
| 0 | `Unverified` | Profile created |
| 1 | `VerifiedIdentity` | Validator approves identity milestone |
| 2 | `PerformanceMilestones` | Validator approves performance milestone |
| 3 | `EliteTier` | Scout logs trial offer |

---

## Events

| Event | Contract | Emitted When |
|-------|----------|-------------|
| `player_registered` | registration | New player profile created |
| `scout_registered` | registration | New scout profile created |
| `profile_updated` | registration | Player updates IPFS content hashes |
| `milestone_approved` | verification | Validator confirms a player achievement |
| `progress_updated` | progress | Player advances to a new level |
| `scout_subscribed` | scout_access | Scout purchases a subscription |
| `player_contacted` | scout_access | Scout pays to unlock player contact |
| `trial_offer_logged` | scout_access | Scout records a trial offer |
| `fees_withdrawn` | scout_access | Admin withdraws accumulated fees |
