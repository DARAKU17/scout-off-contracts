#!/usr/bin/env bash
# ScoutChain — initialize all deployed contracts
# Run after deploy.sh. Requires .env.contracts to exist.
set -euo pipefail

NETWORK="${1:-testnet}"
# shellcheck source=/dev/null
source .env.contracts

ADMIN="${ADMIN_ADDRESS:?Set ADMIN_ADDRESS}"
DEPLOYER="${DEPLOYER_SECRET:?Set DEPLOYER_SECRET}"
XLM_TOKEN="${XLM_TOKEN_ADDRESS:?Set XLM_TOKEN_ADDRESS}"

# ---------------------------------------------------------------------------
# Guard: verify that DEPLOYER_SECRET is the keypair for ADMIN_ADDRESS.
#
# If the signer and the admin address are different accounts the contracts
# will be initialized with an admin that no one controls, permanently locking
# every admin-gated operation (update_fee_config, withdraw_fees, pause, etc.).
# This check catches the most common mistake — using a throwaway test key in a
# production or shared-testnet deployment.
#
# stellar keys address <secret>  — prints the G-address derived from a secret
# key without any network call.  We compare it against $ADMIN_ADDRESS and abort
# before touching any contract if they differ.
# ---------------------------------------------------------------------------
echo "==> Verifying admin keypair..."
DERIVED_ADMIN=$(stellar keys address "$DEPLOYER" 2>/dev/null || true)

if [[ -z "$DERIVED_ADMIN" ]]; then
  echo "ERROR: Could not derive a public address from DEPLOYER_SECRET." >&2
  echo "       Make sure DEPLOYER_SECRET is a valid Stellar secret key (starts with 'S')." >&2
  exit 1
fi

if [[ "$DERIVED_ADMIN" != "$ADMIN" ]]; then
  echo "ERROR: Keypair mismatch — the signing key does not match ADMIN_ADDRESS." >&2
  echo "       Derived from DEPLOYER_SECRET : $DERIVED_ADMIN" >&2
  echo "       ADMIN_ADDRESS                : $ADMIN" >&2
  echo "" >&2
  echo "  Initializing with a mismatched admin permanently locks all admin operations." >&2
  echo "  Fix .env so that DEPLOYER_SECRET is the secret key for ADMIN_ADDRESS, then" >&2
  echo "  re-run this script." >&2
  exit 1
fi

echo "    OK — signer matches admin address ($ADMIN)"

echo "==> Initializing registration contract..."
stellar contract invoke \
  --id "$REGISTRATION_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$ADMIN"

echo "==> Initializing verification contract..."
stellar contract invoke \
  --id "$VERIFICATION_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$ADMIN"

echo "==> Initializing progress contract..."
stellar contract invoke \
  --id "$PROGRESS_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$ADMIN"

echo "==> Initializing scout_access contract..."
stellar contract invoke \
  --id "$SCOUT_ACCESS_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$ADMIN" \
  --xlm_token "$XLM_TOKEN" \
  --fee_config '{
    "contact_fee_stroops": 1000000,
    "basic_sub_stroops": 10000000,
    "pro_sub_stroops": 30000000,
    "elite_sub_stroops": 70000000,
    "sub_duration_secs": 2592000,
    "pro_contact_limit": 10
  }'

echo "==> Wiring verification → progress cross-contract link..."
stellar contract invoke \
  --id "$VERIFICATION_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- set_progress_contract \
  --progress_contract "$PROGRESS_CONTRACT_ID"

echo "==> Wiring registration ← progress cross-contract link..."
stellar contract invoke \
  --id "$REGISTRATION_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- set_progress_contract \
  --addr "$PROGRESS_CONTRACT_ID"

echo "==> Wiring progress → verification cross-contract link..."
stellar contract invoke \
  --id "$PROGRESS_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- set_verification_contract \
  --addr "$VERIFICATION_CONTRACT_ID"

echo "==> Wiring progress → registration cross-contract link..."
stellar contract invoke \
  --id "$PROGRESS_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- set_registration_contract \
  --addr "$REGISTRATION_CONTRACT_ID"

echo "==> Wiring scout_access → progress cross-contract link..."
stellar contract invoke \
  --id "$SCOUT_ACCESS_CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- set_progress_contract \
  --addr "$PROGRESS_CONTRACT_ID"


echo ""
echo "==> Querying deployed contract versions..."
for entry in \
  "registration:$REGISTRATION_CONTRACT_ID" \
  "verification:$VERIFICATION_CONTRACT_ID" \
  "progress:$PROGRESS_CONTRACT_ID" \
  "scout_access:$SCOUT_ACCESS_CONTRACT_ID"; do
  name="${entry%%:*}"
  id="${entry#*:}"
  version=$(stellar contract invoke \
    --id "$id" \
    --network "$NETWORK" \
    -- version)
  echo "    $name version => $version"
done

echo ""
echo "==> All contracts initialized and wired."
