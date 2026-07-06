#!/usr/bin/env bash
# ScoutChain — generate TypeScript bindings for all contracts
# Usage: ./scripts/generate-bindings.sh [testnet|mainnet]
# Requires .env.contracts to exist (written by deploy.sh)
set -euo pipefail

# Pin the stellar-cli version to ensure reproducible bindings.
# Update this constant and the CI install step together when upgrading.
REQUIRED_STELLAR_CLI_VERSION="21.6.0"

actual_version=$(stellar --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1 || true)
if [[ "$actual_version" != "$REQUIRED_STELLAR_CLI_VERSION" ]]; then
  echo "ERROR: stellar-cli version mismatch."
  echo "       Required: $REQUIRED_STELLAR_CLI_VERSION"
  echo "       Found:    ${actual_version:-<not installed>}"
  echo ""
  echo "Install the correct version:"
  echo "  curl -sSL https://raw.githubusercontent.com/stellar/stellar-cli/v${REQUIRED_STELLAR_CLI_VERSION}/install.sh | bash"
  echo ""
  echo "See docs/CONTRIBUTING.md for setup instructions."
  exit 1
fi

NETWORK="${1:-testnet}"
# shellcheck disable=SC1091
source .env.contracts

CONTRACTS=(registration verification progress scout_access)

declare -A IDS=(
  [registration]="$REGISTRATION_CONTRACT_ID"
  [verification]="$VERIFICATION_CONTRACT_ID"
  [progress]="$PROGRESS_CONTRACT_ID"
  [scout_access]="$SCOUT_ACCESS_CONTRACT_ID"
)

for name in "${CONTRACTS[@]}"; do
  id="${IDS[$name]}"
  out="bindings/${name}"

  echo "==> Generating TypeScript bindings for $name ($id)..."
  stellar contract bindings typescript \
    --contract-id "$id" \
    --network "$NETWORK" \
    --output-dir "$out" \
    --overwrite

  echo "    Written to $out/"
done

echo ""
echo "==> All bindings generated. Publish or link them into backend/frontend."
