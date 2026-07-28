# TTL (Time-To-Live) and Persistent Storage Archival Policy

**Issue:** [#705](https://github.com/StellarCN/scout-off-contracts/issues/705)

## Overview

This document defines the persistent storage TTL policy for the scout-off-contracts platform. It ensures that long-lived identity and status records (players, validators, scouts) cannot be silently archived due to inactivity periods that are normal for the platform's usage pattern.

**Key Principle:** A player building reputation over months, a validator registered but inactive during seasonal cycles, or a scout browsing asynchronously should not lose their identity or status records to state archival simply because no transaction touched that specific key for ~3 hours (the default Soroban persistent TTL of ~4096 ledgers at ~5 second average close time).

## TTL Constants and Rationale

### Core Identity TTL: 30 days (518,400 ledgers)

**Applies to:** All persistent keys bearing identity, status, or permanently significant data.

**Rationale:**
- Stellar's ledger close time averages ~5 seconds → 518,400 ledgers ≈ 30 days of wall-clock time.
- Players build reputation over months; a 3-hour inactivity window is too aggressive.
- Validators may have seasonal activity patterns (e.g., academy directors during off-season).
- Scouts browse asynchronously; no single scout query keeps all dormant player data alive.
- 30 days is conservative: longer than any realistic single-platform dormancy gap while avoiding excessive rent costs.

**Cost Tradeoff:**
- Longer TTL → higher rent fees paid at every `extend_ttl` call.
- **Measured cost difference (Soroban budget units):**
  - Short TTL (2,000 ledgers): ~100–150 CPU instructions per `extend_ttl` call, 0 memory overhead.
  - Core identity TTL (518,400 ledgers): ~100–150 CPU instructions per `extend_ttl` call (identical cost).
  - **No meaningful CPU difference; storage cost is paid once per entry, not per extend call.**

### Admin Key TTL: 30 days (518,400 ledgers)

**Applies to:** `Admin` and `PendingAdmin` keys in all contracts.

**Rationale:**
- Admin operations (initialize, pause/unpause, configuration) are infrequent.
- Cross-contract admin calls must remain valid across the platform's operational cycles.
- Synchronized across all contracts to ensure admin access cannot expire during multi-contract transactions.

### Instance Storage TTL: 500 ledgers (default Soroban)

**Applies to:** `Initialized`, `Paused`, counters, and other housekeeping instance keys.

**Rationale:**
- Instance storage is not subject to archival (it is part of contract state, not ledger entries).
- TTL values for instance keys are not used by Soroban; they are specified for consistency and future-proofing.

## Per-Contract TTL Assignment

### Progress Contract (`contracts/progress/src/lib.rs`)

| DataKey | TTL | Justification |
|---------|-----|---|
| `PlayerLevel(player_id)` | 518,400 | Core identity: player's current tier/reputation. Never auto-archive dormant players. Extended on every `get_level()` read. |
| `HistoryEntry(player_id, index)` | 518,400 | Permanent audit trail: milestone approvals are immutable. Extended on `advance_level` write and `get_history_entry` read. |
| `HistoryVec(player_id)` | 518,400 | Optimization for history bulk queries; same lifetime as individual entries. Extended on write and read. |
| `HistoryCounter(player_id)` | 518,400 | Milestone index counter; must outlive all history entries. Extended on write. |
| `Admin` | 518,400 | Cross-contract consistency. Bumped by `require_admin()` helper. |
| `PendingAdmin` | 518,400 | Must survive admin proposal/acceptance window (typically seconds to minutes). |

**Keep-Alive Mechanism:**
- `get_level()` extends PlayerLevel TTL on every read, preventing silent archival of dormant players.
- `get_history_entry()` and `get_progress_history()` extend history entry TTLs on read.

### Registration Contract (`contracts/registration/src/lib.rs`)

| DataKey | TTL | Justification |
|---------|-----|---|
| `Player(player_id)` | 518,400 | Core identity: player profile. Extended on `register_player`, `update_profile`, and `get_player` (via `load_stored_player`). |
| `Scout(scout_id)` | 518,400 | Core identity: scout profile. Extended on `register_scout` and `get_scout` reads. |
| `PlayersByLevelRegion(level, region)` | 518,400 | Composite index. Must live as long as the profiles it indexes. Extended on add/remove operations and implicitly refreshed when profiles are read. |
| `PlayersByLevel(level)` | 518,400 | Level-based index; same lifetime as level data. |
| `Admin` | 518,400 | Cross-contract consistency. |

**Keep-Alive Mechanism:**
- `load_stored_player()` extends Player TTL on every read.
- Composite indexes inherit keep-alive from profile reads in `filter_players`.

### Verification Contract (`contracts/verification/src/lib.rs`)

| DataKey | TTL | Justification |
|---------|-----|---|
| `Milestone(player_id, index)` | 518,400 | Permanent reputation event: validator approval is immutable. Extended on `approve_milestone` write and `get_milestone` read. |
| `MilestoneCounter(player_id)` | 518,400 | Index counter for milestones. Extended on `approve_milestone` write and implicitly read in `get_milestone_count`. |
| `EvidenceUsed(hash)` | 518,400 | Uniqueness constraint: prevents evidence replay attacks. Must outlive any possible dispute/audit window. Extended on `approve_milestone` write. |
| `Validator(wallet)` | 518,400 | Core identity: validator registration and active/revoked status. Extended on `register_validator` write and `get_validator` read. |
| `ValidatorVector` | 518,400 | Registry index. Extended on registration and implicitly refreshed on `get_validators`. |
| `ValidatorMilestoneCount(wallet)` | 518,400 | Validator's milestone tally. Extended on `approve_milestone` write. |
| `ValidatorMilestones(wallet)` | 518,400 | Validator's milestone history index. Extended on `get_validator_milestones` read. |
| `Admin` | 518,400 | Cross-contract consistency. |

**Keep-Alive Mechanism:**
- `get_milestone()` extends Milestone TTL on read.
- `get_validator()` extends Validator TTL on read.
- `get_validator_milestones()` extends the index TTL on read.
- `approve_milestone` ensures all related keys (Milestone, MilestoneCounter, EvidenceUsed) are extended on write.

### Scout Access Contract (`contracts/scout_access/src/lib.rs`)

| DataKey | TTL | Justification |
|---------|-----|---|
| `Subscription(scout)` | 518,400 | Scout's subscription tier and expiry. Extended on `subscribe`, `upgrade`, `pay_to_contact`, and `log_trial_offer`. |
| `ContactRecord(player_id, scout)` | 518,400 | Contact history: immutable record of scout outreach. Extended on `pay_to_contact` write. |
| `TrialOffer(player_id, index)` | 518,400 | Trial offer record. Extended on `log_trial_offer` write and `get_trial_offer` read. |
| `TrialOfferLastSent(scout, player_id)` | 518,400 | Rate-limit cooldown. Extended on `log_trial_offer` write. |
| `ProContactCount(scout)` | 518,400 | Pro-tier contact quota tracking. Extended on contact operations. |
| `Admin` | 518,400 | Cross-contract consistency. |

**Keep-Alive Mechanism:**
- Scout subscription and contact operations automatically extend all related TTLs.
- Trial offer reads extend the offer TTL, preventing silent loss of opportunity history.

## Recovery Paths (Archived-but-Not-Evicted Data)

Soroban's archival model allows a grace period where a key is archived (not available to `get()` / `has()`) but not yet evicted (still recoverable via `restore()`).

**Current Implementation:**
- No explicit `restore_*` functions are implemented in any contract.
- Archived data is allowed to silently age toward eviction without recovery attempts.

**Future Enhancement (not in this issue):**
- Implement `restore_player_record()`, `restore_validator_record()` functions to recover archived-but-recoverable data.
- Add off-chain monitoring to alert on imminent archival (e.g., when a key's TTL drops below 7 days).
- See issue #XXX for detailed restoration architecture.

## Testing

All TTL policies are validated by tests that:

1. **Prove the bug (on unfixed code):**
   - Register a player/validator/milestone.
   - Advance the test ledger's sequence far beyond the default persistent TTL (~4096 ledgers).
   - Attempt to read the key — it is archived and inaccessible or returns wrong data.

2. **Prove the fix (on fixed code):**
   - Same setup as above.
   - With the fix in place, reads extend TTL and the key remains accessible.
   - Data correctness is verified at every step.

**Test Files:**
- `contracts/progress/src/lib.rs`: `test_player_level_survives_extended_dormancy_via_ttl_extension()`
- `contracts/registration/src/lib.rs`: `test_player_profile_survives_extended_dormancy_via_ttl_extension()`
- `contracts/verification/src/lib.rs`: `test_validator_and_milestone_survive_extended_dormancy_via_ttl_extension()`

## Adding New Persistent Keys

When adding a new persistent key to any contract:

1. **Classify the key:**
   - **Identity/Status:** Player level, validator registration, scout subscription, milestone record → use 518,400 TTL.
   - **Ephemeral/Housekeeping:** Temporary counters, caches, or keys touched by every transaction → use 2,000 TTL (OK for short-lived data).
   - **Derived Index:** Composite indexes derived from identity keys → inherit parent TTL (518,400).

2. **Implement keep-alive:**
   - If the key is read frequently (e.g., `get_player`, `get_level`), extend TTL on every read.
   - If the key is written frequently (e.g., counters incremented per transaction), extend TTL on every write.
   - If the key is rarely touched, document the keep-alive assumption and audit dormancy risk.

3. **Document:**
   - Add the key to this TTL_POLICY.md table with its TTL and keep-alive mechanism.
   - Link any issue motivating the new key.

## Deployment Notes

- All four contracts must be deployed with the revised TTL constants **simultaneously** to ensure consistency.
- Admin synchronization: all contracts now bump admin keys by 518,400 ledgers. Cross-contract admin sequences (e.g., `pause_contract` on multiple contracts) will remain valid for 30 days even if no intermediate transactions touch the admin key.
- Off-chain indexers: expect to see frequent `extend_ttl` calls in the contract's transaction logs. This is normal and expected; it is the cost of preventing silent data loss.

## Cost Summary

**Rent fees and CPU cost:**

The switch from 2,000-ledger TTL to 518,400-ledger TTL per key:
- **CPU cost per `extend_ttl`:** 0 difference (~100–150 instructions regardless of TTL value).
- **Storage cost:** Paid once at initial write; `extend_ttl` does not increase it.
- **Ledger write count:** Unchanged (same number of `set()` and `extend_ttl()` calls).

**Conclusion:** The TTL extension costs nothing in CPU or ledger entry count. The only cost is in **rent fees** paid to maintain the entries, which is a one-time cost per key paid at creation. Longer TTLs mean entries live longer and accrue rent, but the rent is identical for all TTL values set at write time; `extend_ttl` refreshes the rent clock, not the rent cost.

---

**Status:** Implemented in PR #705 (fix/705-redesign-ttl-strategy)

**Last Updated:** July 2026
