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

## Reconciliation

Run `node scripts/reconcile-indexer.js` to compare on-chain state against the
local database.  The script reports:

- Players/scouts present on-chain but missing in the database
- Players/scouts present in the database but missing on-chain
- Field-level mismatches for `players.deactivated` and `scouts.verified`

## Known gaps (resolved)

- ~~`scouts.verified`~~ — column added in migration `001_initial_schema.sql`
- ~~Player deactivation status~~ — column added in migration `001_initial_schema.sql`
