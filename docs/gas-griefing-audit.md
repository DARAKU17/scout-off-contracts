# Gas-Griefing Resistance Audit

## Overview

This document identifies asymmetric-cost patterns where an attacker can cheaply
force expensive work to be paid by another user or the platform. Each confirmed
vector includes a concrete exploit scenario, cost asymmetry estimate, and the
implemented mitigation.

## Confirmed Vectors

### V1: ValidatorVector Monotonic Growth

**Location:** `contracts/verification/src/lib.rs:get_validators` (lines 283–298)

**Mechanism:** `get_validators` loads the full `ValidatorVector` and iterates
every entry, calling `get_validator_status` for each. Revoked validators are
never removed from the vector. As validators churn, every call to
`get_validators` grows linearly with the total number of validators ever
registered, not just active ones.

**Exploit scenario:**
1. Attacker registers 100 validators (cost: 100 cheap `register_validator` txs).
2. Attacker revokes all 100 validators.
3. Every legitimate `get_validators` caller now pays to scan 100 entries.
4. Attacker repeats with new wallets.

**Cost asymmetry:**
- Attacker cost: ~100 × `register_validator` fee (modest)
- Victim cost: every `get_validators` call scans 100 entries, growing over time

**Mitigation:** Maintain a separate `ActiveValidatorVector` that is updated on
registration, revocation, and restoration. `get_validators` reads only the
active vector.

### V2: filter_players Slow-Path Scan Inflation

**Location:** `contracts/registration/src/lib.rs:filter_players` (lines 720–819)

**Mechanism:** When no `region` filter is provided, `filter_players` scans the
full `PlayerIndex`. A high volume of low-quality `register_player` calls
inflates this index, making every unfiltered scout query more expensive.

**Exploit scenario:**
1. Attacker registers 10,000 low-quality players (cost: 10,000 cheap txs).
2. Legitimate scout calls `filter_players` without a region filter.
3. Scout pays to scan all 10,000 entries.

**Cost asymmetry:**
- Attacker cost: ~10,000 × `register_player` fee (modest per entry)
- Victim cost: one `filter_players` call scans 10,000 entries

**Mitigation:** Enforce a hard cap on total registered players
(`MAX_PLAYERS = 10,000`). The slow path is bounded by the cap. This is a
reasonable trade-off: the platform can support 10,000 players before requiring
a contract upgrade to raise the cap.

### V3: TTL-Bump Asymmetry

**Location:** All contracts — every persistent storage write includes TTL bumps.

**Mechanism:** The party calling a function pays for TTL extensions on storage
they did not create. For example, a scout bumping TTL on a `PlayerProfile`
they did not create, or an admin bumping TTL on validator records.

**Exploit scenario:**
1. Attacker creates many storage entries.
2. Legitimate users/operators must pay to maintain those entries via TTL bumps.

**Cost asymmetry:**
- Attacker cost: one-time write + initial TTL
- Victim cost: ongoing TTL extension fees

**Mitigation:** Accepted risk. TTL extension costs are bounded and identical
regardless of TTL value (~100–150 CPU instructions per `extend_ttl`). The
asymmetry is inherent to Soroban's rent model and cannot be eliminated without
a fundamental redesign. Documented as an accepted risk with monitoring.

## Implemented Mitigations

### V1 Mitigation: ActiveValidatorVector

- Added `DataKey::ActiveValidatorVector` to `contracts/verification/src/types.rs`.
- Updated `register_validator`, `revoke_validator`, `restore_validator`, and
  `batch_revoke_validators` to maintain the active vector.
- `get_validators` now reads `ActiveValidatorVector` directly, O(active) instead
  of O(total ever registered).

### V2 Mitigation: MAX_PLAYERS Cap

- Added `MAX_PLAYERS: u32 = 10_000` to `contracts/registration/src/lib.rs`.
- `register_player` now returns `PlayerCapReached` (new error code) when the
  cap is hit.
- This bounds the slow-path scan cost for `filter_players`.

### V3 Mitigation: Accepted Risk with Monitoring

- No code change. Documented in this audit as an accepted asymmetry inherent
  to Soroban's storage model.
- Platform operators should monitor TTL-related CPU costs via existing
  Prometheus metrics.

## Cross-References

- Scalability issue tracking `ValidatorVector` growth: see existing issue #853
  (check-error-code-continuity) and related scalability notes in `docs/TTL_POLICY.md`.
- `filter_players` performance: bounded by `MAX_RESULTS = 50` per call, but
  slow-path scan cost is proportional to total players. Mitigated by `MAX_PLAYERS`.
