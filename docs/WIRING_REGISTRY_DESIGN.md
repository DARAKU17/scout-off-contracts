# Cross-Contract Wiring Registry — Design Document

**Issue**: #801  
**Status**: Prototype implemented (see below)

---

## Problem Statement

ScoutChain's four contracts are interconnected by five independent peer-address
pointer fields:

| Contract | Setter | Storage Key | Re-wiring guard |
|----------|--------|-------------|-----------------|
| `verification` | `set_progress_contract` | `ProgressContract` | First-call-only (`AlreadyConfigured`); use `update_progress_contract` after |
| `registration` | `set_progress_contract` | `ProgressContract` | None — freely re-settable |
| `progress` | `set_verification_contract` | `VerificationContract` | None |
| `progress` | `set_registration_contract` | `RegistrationContract` | None |
| `progress` | `set_scout_access_contract` | `ScoutAccessContract` | None |
| `scout_access` | `set_progress_contract` | `ProgressContract` | None |

Today there is **no on-chain mechanism** to ask "are all five links mutually
consistent?" — `scripts/verify-cross-contract-wiring.sh` polls each contract
externally, but it can only confirm the contracts are alive; it cannot yet read
the stored peer addresses (no public getter functions exist for them).

The asymmetric re-wiring guards (first-call-only vs freely re-settable) are
themselves a documented source of confusion in `docs/DEPLOYMENT.md`.

---

## Approach Comparison

### Option A — Shared-types helper only (no new contract)

Each contract exposes a `get_wiring_state()` public getter that returns a
struct listing all peer addresses it holds. A new `check_wiring_consistency()`
function in `shared-types` can be called by any off-chain caller or CI script
to compute a consistency snapshot without any on-chain coordination.

**Pros:**
- Zero extra cross-contract hops at runtime — no gas cost increase.
- No new contract to deploy or upgrade.
- Fully backward-compatible with already-deployed testnet contracts (add getters
  in the next upgrade, old logic unchanged).
- Simple mental model: "call `get_wiring_state()` on each contract and compare."

**Cons:**
- The consistency check is still an off-chain step — contracts cannot enforce
  wiring correctness within a transaction.
- Four separate `get_wiring_state()` calls are needed for a full picture.

### Option B — Dedicated registry contract

A fifth `WiringRegistry` contract holds all five peer addresses as the single
source of truth. All four contracts query the registry at runtime via
cross-contract call to resolve peer addresses instead of reading local storage.

**Pros:**
- On-chain consistency — updating one entry in the registry is instantly
  visible to all four contracts on the next call.
- Admin workflow is centralised in one contract.

**Cons:**
- **+1 cross-contract hop** on every `approve_milestone`, `subscribe`,
  `batch_contact_players`, and `advance_level` call — at minimum one extra
  `invoke_contract` per transaction, which adds ~500–2,000 CPU instructions
  and a small ledger-read fee per call.
- Requires deploying and initializing a fifth contract.
- Upgrade complexity increases: all four contracts must be upgraded to use the
  new registry before any can be cut over. A partial upgrade leaves a split
  environment.
- **Not backward-compatible** with already-deployed testnet contracts without a
  coordinated multi-contract migration.

### Recommendation: Option A (shared-types getter approach)

The registry-contract approach's runtime overhead and migration complexity
outweigh its benefits for a system where re-wiring happens once at deployment
(rarely more than a few times over a contract's lifetime). Option A achieves
the same operational visibility at zero runtime cost and is safe to roll out
incrementally across upgrades.

---

## Prototype: `get_wiring_state()` on the Progress Contract

The progress contract holds the most wiring links (three: registration,
verification, scout_access), making it the highest-value starting point.

### Storage schema (existing)

```
DataKey::RegistrationContract  → Address
DataKey::VerificationContract  → Address
DataKey::ScoutAccessContract   → Address
```

### New public getter (prototype — implemented in this PR)

```rust
pub fn get_wiring_state(env: Env) -> ProgressWiringState
```

Returns a `ProgressWiringState` struct:

```rust
#[contracttype]
pub struct ProgressWiringState {
    /// Address of the registration contract, if set.
    pub registration_contract: Option<Address>,
    /// Address of the verification contract, if set.
    pub verification_contract: Option<Address>,
    /// Address of the scout_access contract, if set.
    pub scout_access_contract: Option<Address>,
}
```

### Consistency check helper

`ProgressWiringState::is_fully_wired()` returns `true` iff all three addresses
are `Some(_)`. The external verification script uses this to report incomplete
wiring without needing to enumerate storage keys manually.

---

## Updated Verification Script

`scripts/verify-cross-contract-wiring.sh` is updated to call
`get_wiring_state()` on the progress contract and report each link
individually, replacing the current health-only poll that cannot inspect
actual peer addresses.

The script still falls back to health-only checks for contracts that have not
yet been upgraded to expose `get_wiring_state()`.

---

## Migration Note

### For already-deployed testnet contracts

1. **No immediate action required.** The existing five-pointer model continues
   to work identically. The new `get_wiring_state()` getter is additive and
   backward-compatible.

2. **To gain on-chain visibility**, upgrade the progress contract with the new
   WASM (which adds `get_wiring_state()`). The stored wiring addresses are
   preserved across WASM upgrades — the upgrade only adds a new read-only
   function.

3. **Gradually add `get_wiring_state()` to the other three contracts** in
   subsequent minor upgrades. The external verification script handles missing
   getters gracefully by falling back to health checks.

4. **Homogenise re-wiring guards** in a future upgrade: replace the current
   mix of first-call-only vs freely-re-settable with a single consistent policy
   (e.g. admin-only, freely re-settable, with an event emitted on each change).
   This is a separate PR — the current PR only adds the observability layer.

### Timeline

| Step | PR / Upgrade | Scope |
|------|-------------|-------|
| 1 (this PR) | feat/797-798-801-835 | Add `get_wiring_state()` to progress; update verification script |
| 2 (next) | feat/wiring-getters | Add `get_wiring_state()` to registration, verification, scout_access |
| 3 (future) | feat/wiring-policy | Homogenise re-wiring guards across all four contracts |

---

## Open Questions

1. Should `get_wiring_state()` be added to `shared-types` as a trait so the
   pattern is enforced at compile time? Deferred to Step 2.

2. Should the progress contract's `advance_level` verify that its caller's
   address matches the stored verification/scout_access contract address? It
   already does this via `require_auth()`. The wiring state is therefore already
   enforced at call time — the registry approach would not add additional
   security here.
