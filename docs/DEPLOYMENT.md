# Deployment Guide

## Prerequisites

- Rust + `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Stellar CLI: https://developers.stellar.org/docs/tools/developer-tools/cli/install-stellar-cli
- A funded Stellar keypair for deployment

## Step-by-step

### 1. Configure environment

```bash
cp .env.example .env
# Fill in DEPLOYER_SECRET, ADMIN_ADDRESS, XLM_TOKEN_ADDRESS
```

### 2. Deploy all contracts

```bash
chmod +x scripts/deploy.sh
./scripts/deploy.sh testnet
# Contract IDs written to .env.contracts
```

### 3. Initialize and wire contracts

```bash
chmod +x scripts/initialize.sh
./scripts/initialize.sh testnet
# Sets admin, fee config, and wires all cross-contract links:
# - Verification → Progress: verification.set_progress_contract
# - Registration ← Progress: registration.set_progress_contract
# - Progress → Verification: progress.set_verification_contract
# - Progress → Registration: progress.set_registration_contract
# - Scout Access → Progress: scout_access.set_progress_contract
```

### 4. Generate TypeScript bindings

```bash
chmod +x scripts/generate-bindings.sh
./scripts/generate-bindings.sh testnet
# Bindings written to bindings/{contract}/
```

### 5. Seed testnet with demo data (optional)

```bash
chmod +x testnet/seed.sh
./testnet/seed.sh
```

### 6. Run the database migration

Copy `migrations/001_initial_schema.sql` to your backend repo and run it against PostgreSQL:

```bash
psql $DATABASE_URL -f migrations/001_initial_schema.sql
```

### 7. Seed migrated state (optional)

For fresh deployments of an existing production dataset, use the admin-only
seeding entrypoints to replay exported player/scout profiles without requiring
their wallet signatures:

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- admin_seed_player \
  --player_id <id> \
  --wallet <G-address> \
  --vitals '{"age":25,"position":"Forward","region":"Europe","nationality":"FR"}' \
  --ipfs_hashes '["QmHash"]' \
  --registered_at <unix_ts> \
  --level <0-3>

stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- admin_seed_scout \
  --scout_id <id> \
  --wallet <G-address> \
  --region "Europe" \
  --verified false \
  --registered_at <unix_ts>
```

> **Warning:** These functions bypass wallet authentication.  They should only
> be used during a controlled migration replay before the contract serves any
> real wallet-signed registrations.

### 8. Verify indexer consistency

```bash
node scripts/reconcile-indexer.js
```

The script compares on-chain state against the local database and reports
discrepancies for `players.deactivated` and `scouts.verified`.

## Mainnet checklist

- [ ] Audit all four contracts
- [ ] Replace testnet XLM token address with mainnet address in `.env`
- [ ] Set `STELLAR_NETWORK=mainnet` and update RPC/Horizon URLs
- [ ] Run `./scripts/deploy.sh mainnet`
- [ ] Run `./scripts/initialize.sh mainnet`
- [ ] Verify all contract IDs in `.env.contracts`
- [ ] Regenerate bindings: `./scripts/generate-bindings.sh mainnet`

## Upgrading a Deployed Contract

All four contracts expose an `upgrade(new_wasm_hash)` function (admin auth required). The admin address is stored in **persistent** storage so it survives the WASM swap.

Instance storage (Initialized, Paused, counters, fee config) is **not** automatically carried over. You must re-apply it after the upgrade if those values need to be preserved.

### Upgrade procedure

**Step 1 — Read current instance state** (before upgrading)

```bash
# Save values you need to restore
stellar contract invoke --id $CONTRACT_ID -- get_fee_config   # scout_access only
```

**Step 2 — Build and upload the new WASM**

```bash
stellar contract build
stellar contract install \
  --source $DEPLOYER_SECRET \
  --network testnet \
  --wasm target/wasm32v1-none/release/<contract_name>.wasm
# Prints the new wasm hash: <NEW_WASM_HASH>
```

**Step 3 — Call `upgrade`** (must be called by the admin address)

```bash
stellar contract invoke \
  --id $CONTRACT_ID \
  --source $ADMIN_ADDRESS \
  --network testnet \
  -- upgrade \
  --new_wasm_hash <NEW_WASM_HASH>
```

**Step 4 — Re-apply instance state** (if needed)

For `scout_access`, restore fee config:

```bash
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- update_fee_config --fee_config '...'
```

For `verification`, re-wire the progress contract link:

```bash
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  --source $ADMIN_ADDRESS --network testnet \
  -- set_progress_contract \
  --progress_contract $PROGRESS_CONTRACT_ID
```

**Step 5 — Verify**

```bash
stellar contract invoke --id $CONTRACT_ID -- health
```

### What survives an upgrade

| Data | Storage | Survives upgrade? |
|------|---------|-------------------|
| Admin address | Persistent | ✅ Yes |
| Player / scout profiles | Persistent | ✅ Yes |
| Validator registry | Persistent | ✅ Yes |
| Milestone / subscription records | Persistent | ✅ Yes |
| Initialized flag | Instance | ⚠️ Must re-set if wiped |
| Paused flag | Instance | ⚠️ Must re-set if wiped |
| Fee config (scout_access) | Instance | ⚠️ Must re-set |
| XLM token address (scout_access) | Instance | ⚠️ Must re-set |
| Progress contract link (verification) | Instance | ⚠️ Must re-wire |

> **Note:** On Stellar, instance storage is **not** automatically wiped during an `upgrade()` call — only the contract code (WASM) is replaced. The table above reflects the risk if the new WASM changes the storage layout or if instance TTL expires. Always re-verify instance state after an upgrade.

## Common Mistakes

**Milestones approved but player levels don't advance**
You skipped the cross-contract wiring step. `approve_milestone` calls `advance_level` on the progress contract, but only if all links have been set. Fix it by running:

```bash
./scripts/initialize.sh testnet
```

Or manually:

```bash
# 1. Verification → Progress link
stellar contract invoke --id $VERIFICATION_CONTRACT_ID \
  -- set_progress_contract \
  --progress_contract $PROGRESS_CONTRACT_ID

# 2. Registration ← Progress link
stellar contract invoke --id $REGISTRATION_CONTRACT_ID \
  -- set_progress_contract \
  --addr $PROGRESS_CONTRACT_ID

# 3. Progress → Verification link
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- set_verification_contract \
  --addr $VERIFICATION_CONTRACT_ID

# 4. Progress → Registration link
stellar contract invoke --id $PROGRESS_CONTRACT_ID \
  -- set_registration_contract \
  --addr $REGISTRATION_CONTRACT_ID

# 5. Scout Access → Progress link
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID \
  -- set_progress_contract \
  --addr $PROGRESS_CONTRACT_ID
```

This must be done once after every fresh deployment.
