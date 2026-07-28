#!/usr/bin/env bash
# ScoutChain — full post-deploy readiness check
#
# Combines health-check.sh (init/pause status for all four contracts) and
# verify-cross-contract-wiring.sh (all five cross-contract wiring links) into
# a single pass/fail command.  Run this after every deployment or upgrade to
# confirm all contracts are healthy and correctly wired before routing traffic.
#
# Usage:
#   ./scripts/full-readiness-check.sh [testnet|mainnet|local]
#
# Prerequisites:
#   • .env.contracts must exist (written by deploy.sh)
#   • stellar-cli must be on PATH
#
# Exit codes:
#   0  — all health and wiring checks passed
#   1  — one or more checks failed (see summary table for details)
#
# See also:
#   scripts/health-check.sh              — health-only variant
#   scripts/verify-cross-contract-wiring.sh — wiring-only variant
#   docs/DEPLOYMENT.md                   — deployment guide and post-deploy checklist

set -euo pipefail

NETWORK="${1:-testnet}"

# ---------------------------------------------------------------------------
# Load contract IDs from .env.contracts
# ---------------------------------------------------------------------------
if [[ ! -f .env.contracts ]]; then
    echo "ERROR: .env.contracts not found — did you run deploy.sh?" >&2
    exit 1
fi
# shellcheck source=/dev/null
source .env.contracts

for var in REGISTRATION_CONTRACT_ID VERIFICATION_CONTRACT_ID PROGRESS_CONTRACT_ID SCOUT_ACCESS_CONTRACT_ID; do
    if [[ -z "${!var:-}" ]]; then
        echo "ERROR: $var is not set in .env.contracts — did you run deploy.sh?" >&2
        exit 1
    fi
done

# ---------------------------------------------------------------------------
# Result tracking
# ---------------------------------------------------------------------------
PASS=0
FAIL=0
WARN=0

# Arrays to accumulate per-check results for the combined summary table.
# Each entry is: "STATUS|CHECK_NAME|DETAIL"
declare -a RESULTS=()

record_pass() {
    local check_name="$1"
    local detail="${2:-}"
    PASS=$((PASS + 1))
    RESULTS+=("PASS|${check_name}|${detail}")
}

record_fail() {
    local check_name="$1"
    local detail="${2:-}"
    FAIL=$((FAIL + 1))
    RESULTS+=("FAIL|${check_name}|${detail}")
}

record_warn() {
    local check_name="$1"
    local detail="${2:-}"
    WARN=$((WARN + 1))
    RESULTS+=("WARN|${check_name}|${detail}")
}

invoke() {
    stellar contract invoke \
        --id "$1" \
        --network "$NETWORK" \
        -- "$2" 2>&1
}

echo "============================================"
echo "  ScoutChain Full Readiness Check"
echo "  Network: $NETWORK"
echo "============================================"

# ===========================================================================
# SECTION 1 — Health checks (init/pause status for all four contracts)
# Mirrors the logic in scripts/health-check.sh without calling it as a
# subprocess, so output can be incorporated into the combined summary table.
# ===========================================================================
echo ""
echo "--- Section 1: Contract health (initialized & not paused) ---"

declare -A CONTRACT_IDS=(
    [registration]="$REGISTRATION_CONTRACT_ID"
    [verification]="$VERIFICATION_CONTRACT_ID"
    [progress]="$PROGRESS_CONTRACT_ID"
    [scout_access]="$SCOUT_ACCESS_CONTRACT_ID"
)

HEALTH_CONTRACT_ORDER=(registration verification progress scout_access)

for name in "${HEALTH_CONTRACT_ORDER[@]}"; do
    id="${CONTRACT_IDS[$name]}"
    echo "==> health() on ${name} (${id})..."
    check_label="health:${name}"

    if response=$(invoke "$id" health 2>&1); then
        echo "    Response: $response"

        if echo "$response" | grep -q '"initialized":false'; then
            echo "    ❌ FAIL: ${name} returned initialized: false"
            record_fail "$check_label" "initialized: false — run initialize.sh"
        elif echo "$response" | grep -q '"paused":true'; then
            echo "    ❌ FAIL: ${name} returned paused: true"
            record_fail "$check_label" "paused: true — call unpause_contract to resume"
        elif echo "$response" | grep -q '"initialized":true'; then
            echo "    ✅ OK: ${name} is healthy"
            record_pass "$check_label" "initialized: true, paused: false"
        else
            echo "    ❌ FAIL: ${name} returned unexpected health response"
            record_fail "$check_label" "unexpected health response: ${response}"
        fi
    else
        echo "    ❌ FAIL: ${name} health() call failed — ${response}"
        record_fail "$check_label" "health() invocation failed: ${response}"
    fi
done

# ===========================================================================
# SECTION 2 — Cross-contract wiring verification
# Mirrors the logic in scripts/verify-cross-contract-wiring.sh without calling
# it as a subprocess, so results feed the combined summary table.
# ===========================================================================
echo ""
echo "--- Section 2: Cross-contract wiring (all 5 links) ---"

# ---------------------------------------------------------------------------
# 2a. get_wiring_state() on the progress contract
#     (covers 3 of the 5 links: registration, verification, scout_access)
# ---------------------------------------------------------------------------
echo ""
echo "  Checking progress contract wiring state..."

if state=$(invoke "$PROGRESS_CONTRACT_ID" get_wiring_state 2>&1); then
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

    # Link 1: progress → registration_contract
    if [[ "$reg_addr" == "NOT SET" || "$reg_addr" == "parse_error" ]]; then
        echo "  ❌ FAIL: progress → registration_contract: NOT SET"
        record_fail "wiring:progress→registration" "NOT SET — run: stellar contract invoke --id \$PROGRESS_CONTRACT_ID -- set_registration_contract --addr \$REGISTRATION_CONTRACT_ID"
    elif [[ "$reg_addr" == "$REGISTRATION_CONTRACT_ID" ]]; then
        echo "  ✅ OK: progress → registration_contract: ${reg_addr}"
        record_pass "wiring:progress→registration" "${reg_addr} matches REGISTRATION_CONTRACT_ID"
    else
        echo "  ❌ FAIL: progress → registration_contract: ${reg_addr} ≠ expected ${REGISTRATION_CONTRACT_ID}"
        record_fail "wiring:progress→registration" "${reg_addr} ≠ expected ${REGISTRATION_CONTRACT_ID}"
    fi

    # Link 2: progress → verification_contract
    if [[ "$ver_addr" == "NOT SET" || "$ver_addr" == "parse_error" ]]; then
        echo "  ❌ FAIL: progress → verification_contract: NOT SET"
        record_fail "wiring:progress→verification" "NOT SET — run: stellar contract invoke --id \$PROGRESS_CONTRACT_ID -- set_verification_contract --addr \$VERIFICATION_CONTRACT_ID"
    elif [[ "$ver_addr" == "$VERIFICATION_CONTRACT_ID" ]]; then
        echo "  ✅ OK: progress → verification_contract: ${ver_addr}"
        record_pass "wiring:progress→verification" "${ver_addr} matches VERIFICATION_CONTRACT_ID"
    else
        echo "  ❌ FAIL: progress → verification_contract: ${ver_addr} ≠ expected ${VERIFICATION_CONTRACT_ID}"
        record_fail "wiring:progress→verification" "${ver_addr} ≠ expected ${VERIFICATION_CONTRACT_ID}"
    fi

    # Link 3: progress → scout_access_contract
    if [[ "$sa_addr" == "NOT SET" || "$sa_addr" == "parse_error" ]]; then
        echo "  ❌ FAIL: progress → scout_access_contract: NOT SET"
        record_fail "wiring:progress→scout_access" "NOT SET — run: stellar contract invoke --id \$PROGRESS_CONTRACT_ID -- set_scout_access_contract --addr \$SCOUT_ACCESS_CONTRACT_ID"
    elif [[ "$sa_addr" == "$SCOUT_ACCESS_CONTRACT_ID" ]]; then
        echo "  ✅ OK: progress → scout_access_contract: ${sa_addr}"
        record_pass "wiring:progress→scout_access" "${sa_addr} matches SCOUT_ACCESS_CONTRACT_ID"
    else
        echo "  ❌ FAIL: progress → scout_access_contract: ${sa_addr} ≠ expected ${SCOUT_ACCESS_CONTRACT_ID}"
        record_fail "wiring:progress→scout_access" "${sa_addr} ≠ expected ${SCOUT_ACCESS_CONTRACT_ID}"
    fi
else
    echo "  ⚠️  progress: get_wiring_state() not available — contract may need upgrading."
    echo "     See docs/WIRING_REGISTRY_DESIGN.md — Step 1 of the wiring observability rollout."
    record_warn "wiring:progress→registration" "get_wiring_state() not available on progress contract"
    record_warn "wiring:progress→verification" "get_wiring_state() not available on progress contract"
    record_warn "wiring:progress→scout_access" "get_wiring_state() not available on progress contract"
fi

# ---------------------------------------------------------------------------
# 2b. Remaining links not yet exposed via get_wiring_state()
#     (verification → progress, scout_access → progress)
#     Report as pending Step 2 upgrade, mirroring verify-cross-contract-wiring.sh.
# ---------------------------------------------------------------------------
echo ""
echo "  Remaining links (pending Step 2 upgrade — getter not yet on these contracts):"
echo "  ⚠️  verification → progress_contract: getter not yet available on this contract."
echo "  ⚠️  scout_access  → progress_contract: getter not yet available on this contract."
echo "  Add get_wiring_state() to these contracts (see docs/WIRING_REGISTRY_DESIGN.md §Step 2)"
echo "  and re-run this script to verify all five links."
record_warn "wiring:verification→progress" "getter not yet available — pending Step 2 upgrade (docs/WIRING_REGISTRY_DESIGN.md)"
record_warn "wiring:scout_access→progress"  "getter not yet available — pending Step 2 upgrade (docs/WIRING_REGISTRY_DESIGN.md)"

# ===========================================================================
# COMBINED SUMMARY TABLE
# ===========================================================================
echo ""
echo "============================================"
echo "  Full Readiness Check — Combined Summary"
echo "  Network: $NETWORK"
echo "============================================"
printf "  %-42s  %-6s  %s\n" "CHECK" "STATUS" "DETAIL"
printf "  %-42s  %-6s  %s\n" "$(printf '%0.s-' {1..42})" "------" "------"

for entry in "${RESULTS[@]}"; do
    IFS='|' read -r status check_name detail <<< "$entry"
    case "$status" in
        PASS) icon="✅ PASS" ;;
        FAIL) icon="❌ FAIL" ;;
        WARN) icon="⚠️  WARN" ;;
        *)    icon="$status" ;;
    esac
    printf "  %-42s  %-6s  %s\n" "$check_name" "$icon" "$detail"
done

echo ""
echo "  Totals: ${PASS} passed, ${FAIL} failed, ${WARN} warnings"
echo "============================================"

# ---------------------------------------------------------------------------
# Exit with failure if any check failed, clearly naming the cause.
# Warnings (pending Step 2 upgrades) do not cause a non-zero exit.
# ---------------------------------------------------------------------------
if [[ "$FAIL" -gt 0 ]]; then
    echo ""
    echo "  RESULT: ❌ READINESS CHECK FAILED"
    echo ""
    echo "  The following checks failed:"
    for entry in "${RESULTS[@]}"; do
        IFS='|' read -r status check_name detail <<< "$entry"
        if [[ "$status" == "FAIL" ]]; then
            echo "    ❌ ${check_name}: ${detail}"
        fi
    done
    echo ""
    echo "  Fix the issues above, then re-run: ./scripts/full-readiness-check.sh ${NETWORK}"
    exit 1
fi

echo ""
echo "  RESULT: ✅ ALL CHECKS PASSED"
if [[ "$WARN" -gt 0 ]]; then
    echo "  (${WARN} warning(s) — pending Step 2 wiring upgrades; see docs/WIRING_REGISTRY_DESIGN.md)"
fi
