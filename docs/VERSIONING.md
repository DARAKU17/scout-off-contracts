# Versioning and Upgrade Policy

This document defines what constitutes a breaking change in ScoutChain's
Soroban smart contracts and how upgrades are classified, reviewed, and
executed safely.

---

## Breaking-Change Classification

### Storage-layout breaking changes

A change is a **breaking storage-layout change** if it makes existing
persistent-storage entries unreadable or causes deserialization to fail
after a WASM upgrade. The following changes are always breaking:

1. **A `#[contracttype]` struct or enum used in storage or function signatures
   gains or loses a field** — Soroban serialises structs positionally; adding
   a field changes the byte layout of every existing stored value.

2. **A `#[contracttype]` enum variant is reordered** — variants are serialised
   by discriminant index; reordering changes the numeric mapping.

3. **A field's type changes** — even a widening change (e.g. `u32` → `u64`)
   alters the serialized byte count.

4. **The `DataKey` enum gains, loses, or reorders variants** — the key enum is
   also a `#[contracttype]`; changing it makes existing storage entries
   unreadable by the key that was used to write them.

### Safe (non-breaking) storage-layout changes

The following changes are **safe** and do not require a data migration:

- Adding a new variant to the end of a `DataKey` enum (existing keys
  are unaffected).
- Adding a new `#[contracttype]` struct or enum that is not yet written to
  storage.
- Changing business logic inside a function body without altering types.
- Adding new `pub fn` to a `#[contractimpl]` block.
- Removing a `pub fn` from a `#[contractimpl]` block (existing storage
  entries are unaffected).

---

## Upgrade Checklist

Run through every item before submitting an upgrade PR and again before
executing `scripts/upgrade.sh` against a live network.

### Pre-upgrade

- [ ] **Automated storage-layout check** — run
  `bash scripts/check-storage-layout-compat.sh <old-ref> <new-ref>` and
  confirm it exits 0, or explicitly acknowledge every breaking change it
  reports before proceeding.  This check replaces the previous manual
  self-certification step and is enforced as a hard stop in `upgrade.sh`.

- [ ] Identify every `#[contracttype]` struct/enum and `DataKey` enum that
  changed between the last deployed version and the new build.

- [ ] For each change, classify it using the rules above (breaking vs safe).

- [ ] If any breaking change is detected without the
  `--acknowledge-breaking-change` flag, `upgrade.sh` will abort.  Do not
  bypass this gate without migrating or draining the affected storage first.

- [ ] Confirm the new WASM has been tested on testnet with the exact same
  storage state that production holds (seed from a production snapshot if
  possible).

- [ ] Verify the admin key for the target contract is accessible and that
  `DEPLOYER_SECRET` / `ADMIN_ADDRESS` in `.env` are correct for the target
  network.

### Upgrade execution

- [ ] Run `./scripts/upgrade.sh [testnet|mainnet] <contract-name>`.
  The script will execute `check-storage-layout-compat.sh` automatically.

- [ ] Confirm the on-chain `version()` query returns the expected new version
  after the upgrade.

- [ ] Smoke-test the critical path (register → approve_milestone →
  advance_level) against the upgraded contract.

### Post-upgrade

- [ ] Tag the commit with the deployed version (`vX.Y.Z`).
- [ ] Update `config/testnet.json` or `config/mainnet.json` if contract IDs
  changed (re-deploy rather than upgrade).
- [ ] Notify downstream teams (backend indexer, frontend) of any API changes.

---

## Semantic Versioning

| Change type | Version bump |
|-------------|--------------|
| Breaking storage-layout change or removed `pub fn` | Major (`X.0.0`) |
| New `pub fn`, new safe DataKey variant, new event | Minor (`X.Y.0`) |
| Bug fix, internal refactor, doc update | Patch (`X.Y.Z`) |

---

## Storage Layout Compatibility Checker

`scripts/check-storage-layout-compat.sh` is the mechanical enforcement of
the rules above.  It compares two Rust source references (git refs or file
paths) and reports every `DataKey` and `#[contracttype]` change, classified
as **safe** or **breaking**.

### Usage

```bash
# Compare current working tree against the last tagged release
bash scripts/check-storage-layout-compat.sh HEAD~1 HEAD

# Compare two specific git refs
bash scripts/check-storage-layout-compat.sh v1.0.0 v1.1.0

# Compare two explicit file paths (useful in CI with checked-out artefacts)
bash scripts/check-storage-layout-compat.sh \
  path/to/old/types.rs \
  path/to/new/types.rs
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | No breaking changes detected (or all acknowledged) |
| 1 | One or more breaking changes detected; upgrade blocked |
| 2 | Usage error (wrong number of arguments) |

### Override

If a breaking change is intentional (migration has been prepared), pass
`--acknowledge-breaking-change` to both this script and `upgrade.sh`:

```bash
bash scripts/check-storage-layout-compat.sh HEAD~1 HEAD --acknowledge-breaking-change
./scripts/upgrade.sh mainnet verification --acknowledge-breaking-change
```

This flag records an explicit acknowledgement in the upgrade log but does not
suppress the diagnostic output — operators can always see what changed.
