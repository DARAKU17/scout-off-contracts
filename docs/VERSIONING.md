# Contract Versioning Policy

## Semantic Versioning

ScoutChain contracts follow [Semantic Versioning 2.0.0](https://semver.org/) — `MAJOR.MINOR.PATCH`:

| Component | Incremented when |
|-----------|-----------------|
| **MAJOR** | Breaking change — storage layout changed, function removed, error codes renumbered, event schema changed |
| **MINOR** | Backward-compatible addition — new function, new event, new error code appended at end of enum |
| **PATCH** | Backward-compatible fix — bug fix, gas optimisation, documentation update in source |

The current version of all four contracts is **v0.1.0**.

Each contract exposes a `version()` function that returns its current version string:

```bash
stellar contract invoke --id $REGISTRATION_CONTRACT_ID  -- version
stellar contract invoke --id $VERIFICATION_CONTRACT_ID  -- version
stellar contract invoke --id $PROGRESS_CONTRACT_ID      -- version
stellar contract invoke --id $SCOUT_ACCESS_CONTRACT_ID  -- version
```

---

## What Constitutes a Breaking Change

A change is **breaking** (requires a MAJOR bump) if any of the following are true:

- A `pub fn` is renamed or removed from any contract
- A function's parameter list changes (added, removed, or reordered parameters)
- A `#[contracterror]` variant is renumbered or removed
- A `#[contracttype]` struct or enum used in storage or function signatures gains or loses a field
- The storage key layout changes such that existing persistent-storage entries become unreadable
- An on-chain event's topic or data schema changes in a backward-incompatible way
- The cross-contract interface expected by `set_progress_contract` / `set_verification_contract` changes

A change is **non-breaking** (MINOR or PATCH) if:

- A new `pub fn` is added (existing callers are unaffected)
- A new `#[contracterror]` variant is appended at the end of an enum (existing numeric codes unchanged)
- A new event type is added (existing listeners ignore unknown topics)
- Internal helper functions or private storage keys change

---

## Upgrade Checklist

The upgrade procedure is implemented in `scripts/upgrade.sh` (see [docs/DEPLOYMENT.md — Upgrading a Deployed Contract](docs/DEPLOYMENT.md#upgrading-a-deployed-contract) for manual steps).

```bash
./scripts/upgrade.sh <network> <contract_name> <new_wasm_path>
# Example:
./scripts/upgrade.sh testnet verification target/wasm32v1-none/release/scoutchain_verification.wasm
```

### Pre-upgrade

- [ ] Read all BREAKING CHANGES listed in the release notes for the target version
- [ ] Snapshot current on-chain state that lives in **instance** storage (fee config, initialized flag, contract links) — these survive the WASM swap but must be re-verified
- [ ] Check `version()` on all four contracts to confirm the baseline version before upgrade
- [ ] Run `cargo test --workspace` against the new code locally
- [ ] Test the full upgrade flow on testnet before touching mainnet

### During upgrade (per contract)

- [ ] Build and install the new WASM: `stellar contract build && stellar contract install ...`
- [ ] Call `upgrade(new_wasm_hash)` from the admin address
- [ ] Immediately call `health()` to confirm the contract responds

### Post-upgrade

- [ ] Call `version()` on each upgraded contract to confirm the expected new version
- [ ] Re-verify instance storage (fee config, contract links) — re-apply if values were wiped
- [ ] Verify cross-contract wiring: `./scripts/verify-cross-contract-wiring.sh <network>`
- [ ] Re-run cross-contract wiring if any contract was re-deployed from scratch: `./scripts/initialize.sh <network>`
- [ ] Regenerate TypeScript bindings: `./scripts/generate-bindings.sh <network>`
- [ ] Update backend and frontend repos with the new bindings

---

## v0.1.0 → v0.x.0 Migration Notes

This is the initial release. No prior on-chain state exists. The migration path from v0.1.0 to any future v0.x.0 (minor, backward-compatible) release is:

1. **Build the new WASM** for the changed contract(s).
2. **Install and upgrade** each changed contract using the procedure in [DEPLOYMENT.md](docs/DEPLOYMENT.md#upgrading-a-deployed-contract).
3. **Re-verify instance storage** — fee config and contract links are in instance storage and must be confirmed after each WASM swap.
4. **Re-wire cross-contract links** if any contract address changed (i.e., a contract was re-deployed rather than upgraded in-place).
5. **Regenerate bindings** and redeploy the backend/frontend.

### Storage compatibility (v0.1.0 baseline)

All persistent-storage keys in v0.1.0 use the `DataKey` enum defined in each contract's `types.rs`. Any v0.x.0 release that adds new `DataKey` variants is backward-compatible. Any release that **renames or removes** a `DataKey` variant is a breaking change and requires a MAJOR bump plus a data migration script.

### Error code compatibility (v0.1.0 baseline)

Error code assignments for v0.1.0 are fixed as documented in [docs/CONTRACT_REFERENCE.md](docs/CONTRACT_REFERENCE.md). Future minor releases may only **append** new error codes at the end of each enum. SDK consumers should handle unknown error codes gracefully (treat them as unexpected errors and surface to the user).

> **Known gap:** `ScoutAccessError` code 13 is intentionally reserved and will never be assigned. See `contracts/scout_access/src/errors.rs` for the inline explanation.

---

## Version History

### Format & Entry Guidelines

When adding new entries to the Version History table:
- **Contract Scope**: All four contracts (`registration`, `verification`, `progress`, `scout_access`) were initially released together at `v0.1.0`. Future releases may update all contracts in lockstep or target specific contracts individually. Specify the scope in the **Version** column (e.g., `v0.2.0 (all)` or `v0.2.0 (verification)`).
- **SemVer Bump Type**: Explicitly classify each change as `MAJOR` (breaking storage/API change), `MINOR` (backward-compatible feature/event/error addition), or `PATCH` (backward-compatible bug fix/gas optimization) in the **Type** column.
- **Summary**: Provide a concise summary of changes, explicitly calling out breaking changes if `MAJOR`.

| Version | Date | Type | Summary |
|---------|------|------|---------|
| v0.1.0 (all) | 2025 | MINOR | Initial release — all four contracts with full test coverage |
<!-- Template / Example for future entries: -->
<!-- | v0.2.0 (verification) | YYYY-MM-DD | MINOR | Added batch verification helper functions | -->
<!-- | v1.0.0 (all) | YYYY-MM-DD | MAJOR | BREAKING: Updated storage key layout across all contracts | -->
