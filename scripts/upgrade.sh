#!/usr/bin/env bash
# upgrade.sh — Upload a new WASM to an existing deployed ScoutChain contract.
#
# Usage:
#   ./scripts/upgrade.sh [testnet|mainnet] <contract-name> [--acknowledge-breaking-change]
#
# <contract-name> must be one of: registration, verification, progress, scout_access
#
# Pre-upgrade storage-layout check:
#   This script automatically runs scripts/check-storage-layout-compat.sh
#   against the last tagged release and the current HEAD before proceeding.
#   If a breaking storage-layout change is detected the script aborts unless
#   --acknowledge-breaking-change is passed.
#
# See docs/VERSIONING.md for the full upgrade policy and classification rules.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------
NETWORK="${1:-testnet}"
CONTRACT_NAME="${2:-}"
ACKNOWLEDGE_FLAG=""

for arg in "$@"; do
  if [[ "$arg" == "--acknowledge-breaking-change" ]]; then
    ACKNOWLEDGE_FLAG="--acknowledge-breaking-change"
  fi
done

VALID_CONTRACTS=(registration verification progress scout_access)

if [[ -z "$CONTRACT_NAME" ]]; then
  echo "ERROR: contract name required."
  echo "Usage: $0 [testnet|mainnet] <contract-name> [--acknowledge-breaking-change]"
  echo "Valid contracts: ${VALID_CONTRACTS[*]}"
  exit 1
fi

VALID=0
for c in "${VALID_CONTRACTS[@]}"; do
  [[ "$c" == "$CONTRACT_NAME" ]] && VALID=1
done

if [[ $VALID -eq 0 ]]; then
  echo "ERROR: unknown contract '${CONTRACT_NAME}'."
  echo "Valid contracts: ${VALID_CONTRACTS[*]}"
  exit 1
fi

DEPLOYER="${DEPLOYER_SECRET:-}"
if [[ -z "$DEPLOYER" ]]; then
  echo "ERROR: Set DEPLOYER_SECRET env var to your Stellar secret key."
  exit 1
fi

# Load contract IDs from .env.contracts
if [[ -f "${REPO_ROOT}/.env.contracts" ]]; then
  # shellcheck source=/dev/null
  source "${REPO_ROOT}/.env.contracts"
else
  echo "ERROR: .env.contracts not found. Run ./scripts/deploy.sh first."
  exit 1
fi

# Resolve the contract ID variable
CONTRACT_ID_VAR="${CONTRACT_NAME^^}_CONTRACT_ID"
CONTRACT_ID="${!CONTRACT_ID_VAR:-}"
if [[ -z "$CONTRACT_ID" ]]; then
  echo "ERROR: ${CONTRACT_ID_VAR} is not set in .env.contracts."
  exit 1
fi

# Mainnet safety check
if [[ "$NETWORK" == "mainnet" ]]; then
  if grep -q "FILL_IN_BEFORE_USE" "${REPO_ROOT}/config/mainnet.json"; then
    echo "ERROR: config/mainnet.json contains placeholder values."
    echo "Update config/mainnet.json with real values before deploying to mainnet."
    exit 1
  fi
fi

# ---------------------------------------------------------------------------
# Step 1: Storage-layout compatibility check (HARD STOP)
# ---------------------------------------------------------------------------
echo "=== Pre-upgrade storage-layout compatibility check ==="
echo ""

# Find the last git tag to use as the baseline. If none exists, compare HEAD~1.
LAST_TAG=$(git -C "$REPO_ROOT" describe --tags --abbrev=0 2>/dev/null || echo "")
if [[ -z "$LAST_TAG" ]]; then
  echo "  No git tag found — comparing HEAD~1 against HEAD."
  OLD_REF="HEAD~1"
else
  echo "  Comparing ${LAST_TAG} against HEAD."
  OLD_REF="$LAST_TAG"
fi

# shellcheck disable=SC2086
if ! bash "${REPO_ROOT}/scripts/check-storage-layout-compat.sh" \
    "$OLD_REF" HEAD ${ACKNOWLEDGE_FLAG}; then
  echo "Upgrade aborted due to breaking storage-layout changes."
  echo "See docs/VERSIONING.md for how to proceed."
  exit 1
fi

# ---------------------------------------------------------------------------
# Step 2: Build
# ---------------------------------------------------------------------------
WASM_DIR="${REPO_ROOT}/target/wasm32v1-none/release"

echo "==> Building contracts..."
cargo build --workspace --target wasm32v1-none --release

# ---------------------------------------------------------------------------
# Step 3: Optimize
# ---------------------------------------------------------------------------
WASM_SRC="${WASM_DIR}/scoutchain_${CONTRACT_NAME}.wasm"
WASM_OPT="${WASM_DIR}/scoutchain_${CONTRACT_NAME}.optimized.wasm"

echo "==> Optimizing ${CONTRACT_NAME}..."
stellar contract optimize --wasm "$WASM_SRC" --wasm-out "$WASM_OPT"

# ---------------------------------------------------------------------------
# Step 4: Upload WASM and upgrade the live contract
# ---------------------------------------------------------------------------
echo "==> Uploading new WASM for ${CONTRACT_NAME} on ${NETWORK}..."
NEW_WASM_HASH=$(stellar contract upload \
  --wasm "$WASM_OPT" \
  --source "$DEPLOYER" \
  --network "$NETWORK")

echo "    New WASM hash: ${NEW_WASM_HASH}"

echo "==> Invoking upgrade on contract ${CONTRACT_ID}..."
stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- upgrade \
  --new_wasm_hash "$NEW_WASM_HASH"

# ---------------------------------------------------------------------------
# Step 5: Confirm
# ---------------------------------------------------------------------------
echo ""
echo "==> Verifying new version on-chain..."
NEW_VERSION=$(stellar contract invoke \
  --id "$CONTRACT_ID" \
  --source "$DEPLOYER" \
  --network "$NETWORK" \
  -- version 2>/dev/null || echo "version() not available")

echo "    On-chain version: ${NEW_VERSION}"
echo ""
echo "=== Upgrade complete: ${CONTRACT_NAME} on ${NETWORK} ==="
if [[ -n "$ACKNOWLEDGE_FLAG" ]]; then
  echo ""
  echo "NOTE: --acknowledge-breaking-change was passed. A breaking storage-layout"
  echo "      change was accepted. Ensure data migration has been applied."
fi
