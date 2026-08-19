# Migration Gaps

> **This is the canonical reference for migration automation coverage.**  
> When any gap listed here is closed by another issue, update the **Status**
> column and add a note rather than deleting the row — the history of what was
> once non-replayable is useful context for future audits.

Two places in the documentation previously mentioned migration gaps in passing:

- [`docs/DEPLOYMENT.md` — "Address migration"](DEPLOYMENT.md#address-migration-new-contract-id)
  describes what `scripts/replay-state.sh` can and cannot replay automatically.
- [`docs/INDEXER.md` — "Known gaps"](INDEXER.md#known-gaps-between-the-contracts-and-this-schema)
  lists places where the PostgreSQL schema does not track a field the contract exposes.

This document consolidates every known non-fully-automatable migration data
category into one place so a migration operator has a single checklist.

---

## Data-category status table

| # | Data category | Contract | Current status | Notes | Tracking issue |
|---|---------------|----------|---------------|-------|---------------|
| 1 | **Validator registrations** | `verification` | ✅ Fully replayable | `replay-state.sh` re-registers every active validator via the admin-only `register_validator` entrypoint. No user action required. | — |
| 2 | **Player profiles** | `registration` | ✅ Fully replayable | `replay-state.sh` exports player payloads via `get_player_count`/`get_player` and re-seeds them through `admin_seed_player`, which bypasses the wallet-auth requirement of `register_player`. The exported payload includes the resolved `level` field from the progress contract. | — |
| 3 | **Scout profiles** | `registration` | ✅ Fully replayable | Same path as players: `get_scout_count`/`get_scout` → `admin_seed_scout`. | — |
| 4 | **Player progress levels** | `progress` | ✅ Fully replayable | The `level` field is captured in the player export (see row 2) and written via `admin_seed_player` on the new registration contract, then synced to the new progress contract by the cross-contract `set_player_level` call that `admin_seed_player` triggers. | — |
| 5 | **Progress history entries** | `progress` | ⚠️ Partially replayable | `get_history_count`/`get_history_entry` expose the full history, but there is no admin-seed entrypoint to re-write individual `HistoryEntry` records on a new contract. The current script captures a snapshot for audit purposes but does **not** replay history into the new contract. | Open — no tracking issue yet |
| 6 | **Milestone records** | `verification` | ⚠️ Partially replayable | Each approved milestone is readable via `get_milestone_count`/`get_milestone`, but there is no admin-seed entrypoint to replay them onto a new contract. The evidence-hash uniqueness index (`EvidenceUsed`) also cannot be reconstructed without replaying every `approve_milestone` call with validator auth. Indices (MilestoneCounter, GlobalMilestoneIndex) would require careful reconstruction. | Open — no tracking issue yet |
| 7 | **In-flight milestone disputes** | `verification` | ❌ Not replayable | `dispute_milestone` requires player auth, and `resolve_dispute` requires admin auth. There is no admin-seed path for disputes. An active dispute on the old contract has no equivalent on the new contract after migration. Operators must manually resolve or acknowledge open disputes before migrating. | Open — **identified in this documentation audit** |
| 8 | **Scout subscriptions** | `scout_access` | ⚠️ Partially replayable | `get_subscription` exposes the current subscription record per scout. A future `admin_seed_subscription` entrypoint could replay them, but none exists today. XLM balances are on-chain; the contract-held `AccumulatedFees` are replayed via the normal withdrawal path before migration. | Open — no tracking issue yet |
| 9 | **Contact records** | `scout_access` | ⚠️ Partially replayable | `get_player_contacts` / `ScoutContacts` index expose which scouts contacted which players, but there is no admin-seed path. Replaying would require re-invoking `pay_to_contact` or an admin seed entrypoint. | Open — no tracking issue yet |
| 10 | **Trial offers** | `scout_access` | ⚠️ Partially replayable | `get_trial_count`/`get_trial_offer` expose offer records, but there is no admin-seed entrypoint. In-flight trial escrows (unconfirmed, un-expired) are at risk of loss. | Open — no tracking issue yet |
| 11 | **Fee configuration history** | `scout_access` | ⚠️ Partially replayable | The last 5 `FeeConfig` snapshots are on-chain via `get_fee_config_history`. The current config is re-applied by `initialize.sh` on the new contract; the bounded history is not replayed. | Open — cosmetic gap only |
| 12 | **Player deactivation state** | `registration` | ❌ Not tracked in indexer | `registration.deactivate_player` / `reactivate_player` have no `active` column in the PostgreSQL `players` table. A deactivated player appears indistinguishable from an active one in the off-chain index. | Open — indexer schema gap, see [INDEXER.md](INDEXER.md#known-gaps-between-the-contracts-and-this-schema) |
| 13 | **Auto-renewal flags** | `scout_access` | ⚠️ Partially replayable | `get_auto_renew` exposes per-scout opt-in state, but there is no admin-seed path. Scouts would need to re-opt-in after a contract migration. | Open — no tracking issue yet |
| 14 | **In-flight jury votes** | `verification` | ❌ Not replayable | `cast_dispute_vote` requires validator auth; there is no admin-seed path for individual vote records (`DisputeVote` storage). An active jury dispute with votes accumulated on the old contract has no vote-replay path on the new contract after migration. Operators must let active jury votes expire (wait for `voting_deadline` and call `tally_dispute` on the old contract) or acknowledge the loss before migrating. This mirrors the existing "in-flight milestone disputes" gap (row 7). | Open — issue #1036 |

---

## What "status" means

| Status | Meaning |
|--------|---------|
| ✅ Fully replayable | `scripts/replay-state.sh` handles this automatically with no manual steps |
| ⚠️ Partially replayable | Data is readable on-chain but no automated write path exists on the new contract; requires a future admin-seed entrypoint or manual operator action |
| ❌ Not replayable | No automated or admin path exists; requires manual resolution before or after migration |

---

## Migration operator checklist

Before running `scripts/migrate-contract.sh`, verify the following:

1. **Milestone disputes** — run `verification.list_disputes_page(0, 50)` on the
   old contract and confirm all open disputes are resolved or explicitly
   acknowledged as lost. There is currently no replay path (row 7).

2. **In-flight jury votes** — for any jury-required dispute still in its voting
   window, either wait for the deadline and call `tally_dispute` on the old
   contract before migrating, or acknowledge that accumulated votes will be lost
   (row 14). The dispute record itself (with its snapshotted quorum and deadline)
   carries over via the normal dispute replay once an admin-seed path is added,
   but individual vote records do not.

3. **Trial escrows** — run `scout_access.get_trial_count` per active player and
   check for unconfirmed, non-expired `TrialEscrow` entries. Consider calling
   `expire_trial_offers` to clean up stale escrows before migrating (row 10).

3. **Player deactivations** — if `deactivate_player` has ever been called,
   cross-check the off-chain `players` table against on-chain state before
   migrating, since the database has no `active` column (row 12).

4. **Scout subscriptions** — note any subscriptions that expire soon; scouts
   will need to re-subscribe on the new contract (row 8).

5. **Auto-renewal flags** — notify scouts that auto-renewal opt-in state will
   not carry over and they must re-enable it after migration (row 13).

---

## How to update this document

This document is a **living index**. When any gap is closed:

1. Change the **Status** to ✅ and update the **Notes** column with the
   entrypoint or mechanism that now handles it.
2. Add a reference to the PR or issue that closed the gap in **Tracking issue**.
3. Do **not** delete the row — historical context matters for future audits.

Cross-linked from:

- [docs/DEPLOYMENT.md — Address migration](DEPLOYMENT.md#address-migration-new-contract-id)
- [docs/INDEXER.md — Known gaps](INDEXER.md#known-gaps-between-the-contracts-and-this-schema)
