# Indexer Documentation

## Overview

The indexer watches the Soroban RPC event stream and materializes on-chain state
into PostgreSQL for fast querying.  It runs as a separate process from the API.

## What it checks

| Table | Source | Description |
|-------|--------|-------------|
| `players` | `registration.get_player` / `filter_players` | Player profiles, vitals, IPFS hashes |
| `scouts` | `registration.get_scout` | Scout profiles, region, **verified** flag |
| `contact_records` | `scout_access.player_contacted` events | Contact audit trail |
| `trial_offers` | `scout_access.log_trial_offer` events | Trial offer records |
| `subscriptions` | `scout_access.scout_subscribed` events | Active subscriptions |
| `fee_withdrawals` | `scout_access.fees_withdrawn` events | Fee withdrawal audit log |
| `evidence_access_grants` | `scout_access.evidence_access_granted` / `evidence_access_revoked` events | Confidential-evidence viewer authorizations (see [EVIDENCE_PRIVACY.md](EVIDENCE_PRIVACY.md)) |

## Reconciliation

Run `node scripts/reconcile-indexer.js` to compare on-chain state against the
local database.  The script reports:

- Players/scouts present on-chain but missing in the database
- Players/scouts present in the database but missing on-chain
- Field-level mismatches for `players.deactivated` and `scouts.verified`

## Known gaps (resolved)

- ~~`scouts.verified`~~ — column added in migration `001_initial_schema.sql`
- ~~Player deactivation status~~ — column added in migration `001_initial_schema.sql`
# Indexer Reconciliation

`migrations/001_initial_schema.sql` defines the PostgreSQL schema the backend
event indexer mirrors on-chain state into (see
[README.md — Database Schema](../README.md#database-schema)). Nothing forces
that copy to stay accurate: a missed event, a reorg the indexer didn't
rewind for, or a plain indexing bug can all cause silent drift between the
database and the contracts' authoritative on-chain state. Since scout
discovery (`filter_players`) and fee accounting both read from the off-chain
copy, drift there is a real, hard-to-detect data-integrity risk.

`scripts/reconcile-indexer.js` closes that gap: it queries live contract
state and diffs it against the corresponding Postgres rows, table by table.

## When to run it

- **On a schedule** (recommended: hourly or daily via cron/CI) against
  production, to catch indexer drift before it's noticed by a scout or
  player.
- **After any indexer deploy or replay**, to confirm the replay landed
  correctly.
- **Whenever `filter_players` results or fee balances look wrong** — this is
  the first diagnostic step before assuming the contract itself is at fault.
- **After a Stellar network incident** (a ledger reorg, an RPC provider
  outage) that might have caused the indexer to process events out of order
  or twice.

## How to run it

Requires:

- Node.js >= 18 and `npm install` run once at the repo root (`pg` is the only
  dependency).
- The [pinned `stellar-cli` version](CONTRIBUTING.md#installing-the-pinned-stellar-cli-version)
  used by `scripts/generate-bindings.sh`, on your `PATH`.
- A `DATABASE_URL` pointing at the backend's Postgres instance. **This
  database lives in the `scoutchain-backend` repo, not here** — this script
  is intentionally standalone and takes the connection string as a
  parameter rather than assuming any deployment.
- The four `*_CONTRACT_ID` variables, either exported directly or available
  in a `.env.contracts` file (the same file `deploy.sh` writes).

```bash
npm install

DATABASE_URL=postgres://user:pass@host:5432/scoutchain \
REGISTRATION_CONTRACT_ID=C... \
VERIFICATION_CONTRACT_ID=C... \
PROGRESS_CONTRACT_ID=C... \
SCOUT_ACCESS_CONTRACT_ID=C... \
  node scripts/reconcile-indexer.js --network testnet
```

Useful options:

| Flag | Purpose |
|------|---------|
| `--network <name>` | Passed to `stellar contract invoke` (default: `testnet`) |
| `--rpc-url <url>` | Enables the `indexer_cursor` ledger-lag check |
| `--source <identity>` | Passed as `--source` to `stellar contract invoke`, if your CLI setup requires one for simulation calls |
| `--sample <n>` | Cap the number of player/scout IDs walked, for a quick spot-check instead of a full sweep |
| `--tables <a,b,c>` | Check only a subset of tables (see the table list below) |
| `--json` | Emit the report as JSON instead of text, for feeding into another tool |

Exit code is `0` for a clean run and `1` when drift is found — wire this
into a scheduled job (cron, CI) and alert on non-zero.

## What it checks

For each table, the script walks the *authoritative on-chain enumeration*
where one exists (a counter or a full-list getter) rather than only walking
Postgres rows — that way a record the indexer never wrote at all is caught
too, not only value-level drift on rows both sides already agree exist.

| Table | On-chain source | Compared fields |
|-------|-----------------|------------------|
| `players` | `registration.get_player_count` + `get_player`, `progress.get_level` | age, position, region, nationality, ipfs_hashes, level, registered_at, updated_at |
| `scouts` | `registration.get_scout_count` + `get_scout` | wallet, region, registered_at, verified |
| `validators` | DB-driven `verification.get_validator`, cross-checked against `get_validators` (active list) | credentials, active, registered_at, existence |
| `milestones` | `verification.get_milestone_count` + `get_milestone`, per player | validator, description, evidence_hash, approved_at |
| `milestone_disputes` | `verification.has_dispute` / `get_dispute`, tied to the milestone loop | reason, disputed_at, resolved, upheld |
| `scout_subscriptions` | `scout_access.get_subscribers_by_tier` (all three tiers) + `get_subscription` | tier, subscribed_at, expires_at |
| `trial_offers` | `scout_access.get_trial_count` + `get_trial_offer`, per player | scout, details_hash, logged_at |
| `contact_records` | `scout_access.get_player_contacts`, per player | existence only (the contract's `contacted_at` is a ledger timestamp; the DB column records indexer insert time, so it isn't a comparable field — see "Known gaps" below) |
| `evidence_access_grants` | `scout_access.get_player_access_grants` (paginated, 50/page), per player | existence, `granted_at`, `tier_at_grant`, `revoked` |
| `indexer_cursor` | Soroban RPC `getLatestLedger` (only if `--rpc-url` is passed) | reports ledger lag when it exceeds 100 ledgers; informational, not a hard mismatch |

`player_level_history`, `validator_history`, `fee_config_history`, and
`admin_transfers` are pure event logs with no single "current state" getter
to diff against — reconciling them exactly would mean replaying every
emitted event, which is a different tool. The script documents this
explicitly (it prints them under "Skipped" rather than silently omitting
them) and, for `player_level_history`, cross-checks the per-player row count
against `progress.get_history_count` as a cheap drift signal.

## Known gaps between the contracts and this schema

> **For a consolidated list of all migration gaps — including data categories that
> cannot be automatically replayed onto a new contract — see
> [`docs/MIGRATION_GAPS.md`](MIGRATION_GAPS.md).**

These aren't reconciliation failures — they're places the migration doesn't
track a field the contract exposes, discovered while building this tool.
Worth fixing in a future migration if this class of on-chain state is used
by any actual query:

- Player deactivation (`registration.deactivate_player` /
  `reactivate_player`) has no column in `players`.

## What to do when drift is found

1. **Re-run with `--sample`** on the affected table to confirm the drift is
   reproducible and not a transient RPC hiccup.
2. **Check `indexer_cursor` first** — if the indexer is far behind the
   latest ledger, most "mismatches" are just events it hasn't processed yet,
   not real bugs. Wait for it to catch up and re-run.
3. **For a handful of rows**: manually re-derive the correct value from the
   contract and issue a targeted `UPDATE` in the backend's indexer, then
   confirm the fix with `--tables <table> --sample <n>` scoped to the
   affected IDs.
4. **For widespread drift across a table**: treat it as an indexer bug —
   check the backend's event-processing logs around when the drift likely
   started, and consider a full replay of that table from the on-chain
   event log rather than patching rows by hand.
5. **Escalate** to whoever owns the `scoutchain-backend` indexer if the
   cause isn't obvious from the mismatch detail — the report includes the
   exact key, field, on-chain value, and off-chain value needed to start
   that investigation.

## Related documentation

- [RUNBOOK.md](RUNBOOK.md) — emergency pause/unpause and other operational
  procedures.
- [CONTRACT_REFERENCE.md](CONTRACT_REFERENCE.md) — full getter reference for
  every function this script calls.
- [README.md — Database Schema](../README.md#database-schema) — the
  Postgres schema this script reconciles against.
