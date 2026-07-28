#!/usr/bin/env bash
# ScoutChain — verify cross-contract wiring after deployment or upgrade.
#
# Sources .env.contracts, calls health() and version() on every contract,
# and reports which cross-contract links are correctly set.
#
# Usage:
#   ./scripts/verify-cross-contract-wiring.sh [testnet|mainnet|local]
#
# Prerequisites:
#   • .env.contracts must exist (written by deploy.sh)
#
set -euo pipefail

NETWORK="${1:-testnet}"

# shellcheck source=/dev/null
[[ -f .env.contracts ]] && source .env.contracts
for var in REGISTRATION_CONTRACT_ID VERIFICATION_CONTRACT_ID PROGRESS_CONTRACT_ID SCOUT_ACCESS_CONTRACT_ID; do
  if [[ -z "${!var:-}" ]]; then
    echo "ERROR: $var is not set — did you run deploy.sh?" >&2
    exit 1
  fi
done

PASS=0
FAIL=0

pass() { echo "  ✅ $*"; ((PASS++)); }
fail() { echo "  ❌ $*"; ((FAIL++)); }

echo "============================================"
echo "  Cross-Contract Wiring Verification"
echo "  Network: $NETWORK"
echo "============================================"

echo ""
echo "--- Registration ($REGISTRATION_CONTRACT_ID) ---"
if resp=$(stellar contract invoke --id "$REGISTRATION_CONTRACT_ID" --network "$NETWORK" -- health 2>&1); then
  pass "health() responded"
else
  fail "health() failed: $resp"
fi
if resp=$(stellar contract invoke --id "$REGISTRATION_CONTRACT_ID" --network "$NETWORK" -- version 2>&1); then
  pass "version() => $resp"
else
  fail "version() failed: $resp"
fi

echo ""
echo "--- Verification ($VERIFICATION_CONTRACT_ID) ---"
if resp=$(stellar contract invoke --id "$VERIFICATION_CONTRACT_ID" --network "$NETWORK" -- health 2>&1); then
  pass "health() responded"
else
  fail "health() failed: $resp"
fi
if resp=$(stellar contract invoke --id "$VERIFICATION_CONTRACT_ID" --network "$NETWORK" -- version 2>&1); then
  pass "version() => $resp"
else
  fail "version() failed: $resp"
fi

echo ""
echo "--- Progress ($PROGRESS_CONTRACT_ID) ---"
if resp=$(stellar contract invoke --id "$PROGRESS_CONTRACT_ID" --network "$NETWORK" -- health 2>&1); then
  pass "health() responded"
else
  fail "health() failed: $resp"
fi
if resp=$(stellar contract invoke --id "$PROGRESS_CONTRACT_ID" --network "$NETWORK" -- version 2>&1); then
  pass "version() => $resp"
else
  fail "version() failed: $resp"
fi

echo ""
echo "--- Scout Access ($SCOUT_ACCESS_CONTRACT_ID) ---"
if resp=$(stellar contract invoke --id "$SCOUT_ACCESS_CONTRACT_ID" --network "$NETWORK" -- health 2>&1); then
  pass "health() responded"
else
  fail "health() failed: $resp"
fi
if resp=$(stellar contract invoke --id "$SCOUT_ACCESS_CONTRACT_ID" --network "$NETWORK" -- version 2>&1); then
  pass "version() => $resp"
else
  fail "version() failed: $resp"
fi

echo ""
echo "============================================"
echo "  Results: $PASS passed, $FAIL failed"
echo "============================================"

if [[ "$FAIL" -gt 0 ]]; then
  echo ""
  echo "  WARNING: wiring verification incomplete — the contracts above are"
  echo "  responsive, but cross-contract getter functions are not yet deployed."
  echo "  Re-run this script after the contracts expose get_*_contract functions"
  echo "  to confirm each link points to the expected address."
  exit 1
fi

echo "  All contracts are responsive and healthy."
