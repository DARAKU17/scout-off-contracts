#!/usr/bin/env bash
# ScoutChain — verify cross-contract wiring after deployment or upgrade.
#
# Sources .env.contracts, calls health() on every contract to confirm liveness,
# then calls get_wiring_state() on the progress contract (which holds three of
# the five peer-address links) to verify each link is set and consistent.
#
# For contracts that have not yet been upgraded to expose get_wiring_state(),
# the script falls back to health-only checks and reports the links as unverified
# rather than failing outright.
#
# Usage:
#   ./scripts/verify-cross-contract-wiring.sh [testnet|mainnet|local]
#
# Prerequisites:
#   • .env.contracts must exist (written by deploy.sh)
#   • stellar-cli must be on PATH
#
# Exit codes:
#   0  — all checks passed
#   1  — one or more checks failed
#
# See docs/WIRING_REGISTRY_DESIGN.md for the design rationale behind
# get_wiring_state() and the full migration path.

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
WARN=0

pass() { echo "  ✅ $*"; PASS=$((PASS + 1)); }
fail() { echo "  ❌ $*"; FAIL=$((FAIL + 1)); }
warn() { echo "  ⚠️  $*"; WARN=$((WARN + 1)); }

invoke() {
    stellar contract invoke \
        --id "$1" \
        --network "$NETWORK" \
        -- "$2" 2>&1
}

echo "============================================"
echo "  Cross-Contract Wiring Verification"
echo "  Network: $NETWORK"
echo "============================================"

# ---------------------------------------------------------------------------
# 1. Liveness: health() on all four contracts
# ---------------------------------------------------------------------------
echo ""
echo "--- Liveness checks ---"

for label_id in \
    "Registration:$REGISTRATION_CONTRACT_ID" \
    "Verification:$VERIFICATION_CONTRACT_ID" \
    "Progress:$PROGRESS_CONTRACT_ID" \
    "ScoutAccess:$SCOUT_ACCESS_CONTRACT_ID"
do
    label="${label_id%%:*}"
    contract_id="${label_id##*:}"

    if resp=$(invoke "$contract_id" health 2>&1); then
        initialized=$(echo "$resp" | python3 -c "import sys,json; d=json.load(sys.stdin); print('yes' if d.get('initialized') else 'no')" 2>/dev/null || echo "unknown")
        paused=$(echo "$resp"      | python3 -c "import sys,json; d=json.load(sys.stdin); print('yes' if d.get('paused') else 'no')" 2>/dev/null || echo "unknown")
        if [[ "$paused" == "yes" ]]; then
            warn "$label: alive — initialized=$initialized PAUSED=yes"
        else
            pass "$label: alive — initialized=$initialized paused=no"
        fi
    else
        fail "$label ($contract_id): health() failed — $resp"
    fi
done

# ---------------------------------------------------------------------------
# 2. Wiring: get_wiring_state() on the progress contract
#    (prototype for issue #801 — other contracts gain this in a follow-up PR)
# ---------------------------------------------------------------------------
echo ""
echo "--- Wiring state (progress contract) ---"

if state=$(invoke "$PROGRESS_CONTRACT_ID" get_wiring_state 2>&1); then
    # Parse the JSON response for each peer address field
    reg_addr=$(echo "$state" | python3 -c "
import sys, json
d = json.load(sys.stdin)
v = d.get('registration_contract')
print(v if v else 'NOT SET')
" 2>/dev/null || echo "parse_error")

    ver_addr=$(echo "$state" | python3 -c "
import sys, json
d = json.load(sys.stdin)
v = d.get('verification_contract')
print(v if v else 'NOT SET')
" 2>/dev/null || echo "parse_error")

    sa_addr=$(echo "$state" | python3 -c "
import sys, json
d = json.load(sys.stdin)
v = d.get('scout_access_contract')
print(v if v else 'NOT SET')
" 2>/dev/null || echo "parse_error")

    # Check each link
    if [[ "$reg_addr" == "NOT SET" || "$reg_addr" == "parse_error" ]]; then
        fail "progress → registration_contract: NOT SET (run: stellar contract invoke --id \$PROGRESS_CONTRACT_ID -- set_registration_contract --addr \$REGISTRATION_CONTRACT_ID)"
    elif [[ "$reg_addr" == "$REGISTRATION_CONTRACT_ID" ]]; then
        pass "progress → registration_contract: $reg_addr ✓ matches REGISTRATION_CONTRACT_ID"
    else
        fail "progress → registration_contract: $reg_addr ≠ expected $REGISTRATION_CONTRACT_ID"
    fi

    if [[ "$ver_addr" == "NOT SET" || "$ver_addr" == "parse_error" ]]; then
        fail "progress → verification_contract: NOT SET (run: stellar contract invoke --id \$PROGRESS_CONTRACT_ID -- set_verification_contract --addr \$VERIFICATION_CONTRACT_ID)"
    elif [[ "$ver_addr" == "$VERIFICATION_CONTRACT_ID" ]]; then
        pass "progress → verification_contract: $ver_addr ✓ matches VERIFICATION_CONTRACT_ID"
    else
        fail "progress → verification_contract: $ver_addr ≠ expected $VERIFICATION_CONTRACT_ID"
    fi

    if [[ "$sa_addr" == "NOT SET" || "$sa_addr" == "parse_error" ]]; then
        fail "progress → scout_access_contract: NOT SET (run: stellar contract invoke --id \$PROGRESS_CONTRACT_ID -- set_scout_access_contract --addr \$SCOUT_ACCESS_CONTRACT_ID)"
    elif [[ "$sa_addr" == "$SCOUT_ACCESS_CONTRACT_ID" ]]; then
        pass "progress → scout_access_contract: $sa_addr ✓ matches SCOUT_ACCESS_CONTRACT_ID"
    else
        fail "progress → scout_access_contract: $sa_addr ≠ expected $SCOUT_ACCESS_CONTRACT_ID"
    fi
else
    warn "progress: get_wiring_state() not available — contract may need upgrading."
    warn "  See docs/WIRING_REGISTRY_DESIGN.md — Step 1 of the wiring observability rollout."
    warn "  Falling back to health-only check (already reported above)."
fi

# ---------------------------------------------------------------------------
# 3. Remaining links (verification → progress, registration → progress,
#    scout_access → progress) — these contracts do not yet expose
#    get_wiring_state(); report as pending the Step 2 upgrade.
# ---------------------------------------------------------------------------
echo ""
echo "--- Remaining links (pending Step 2 upgrade) ---"
warn "verification → progress_contract: getter not yet available on this contract."
warn "registration → progress_contract: getter not yet available on this contract."
warn "scout_access → progress_contract: getter not yet available on this contract."
echo "  Add get_wiring_state() to these contracts (see docs/WIRING_REGISTRY_DESIGN.md §Step 2)"
echo "  and re-run this script to verify all five links."

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
echo ""
echo "============================================"
echo "  Results: $PASS passed, $FAIL failed, $WARN warnings"
echo "============================================"

if [[ "$FAIL" -gt 0 ]]; then
    echo ""
    echo "  One or more wiring links are broken or inconsistent."
    echo "  Fix the links shown above before routing live traffic to these contracts."
    exit 1
fi

echo ""
echo "  All verified links are correctly wired."
if [[ "$WARN" -gt 0 ]]; then
    echo "  ($WARN warning(s) — see above for pending Step 2 upgrades)"
fi
