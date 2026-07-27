# Contributing

## Setup

```bash
rustup target add wasm32-unknown-unknown
rustup component add clippy rustfmt
cp .env.example .env
```

## Before opening a PR

```bash
cargo test --workspace          # all tests must pass
cargo clippy --workspace        # zero warnings
cargo fmt --all -- --check      # formatting must be clean
bash scripts/check-docs.sh      # docs completeness + cross-contract call drift check
```

## CI checks

| Check | Script / command | What it catches |
|-------|-----------------|-----------------|
| Build & tests | `cargo test --workspace` | Compilation errors, logic regressions |
| Linting | `cargo clippy --workspace -- -D warnings` | Rust anti-patterns |
| Formatting | `cargo fmt --all -- --check` | Style inconsistency |
| Function docs completeness | `bash scripts/check-docs.sh` | Missing `pub fn` entries in `CONTRACT_REFERENCE.md` |
| Error code drift | `bash scripts/check-docs.sh` | Error code/variant mismatches in `CONTRACT_REFERENCE.md` |
| Cross-contract call drift | `bash scripts/check-docs.sh` | Functions that make undocumented cross-contract calls, or doc claims a call the code doesn't make |
| Storage layout compat | `bash scripts/check-storage-layout-compat.sh` | Breaking `#[contracttype]` / `DataKey` changes between versions |
| Bindings validation | `bash scripts/check-bindings.sh` | Malformed TypeScript binding package scaffolds |
| Shell script lint | `shellcheck scripts/*.sh testnet/seed.sh` | Shell script errors and portability issues |

## Contract change checklist

- [ ] New functions have unit tests covering the happy path and at least one error case
- [ ] Any new `DataKey` variant is documented with a comment
- [ ] Cross-contract calls are documented with a `**Cross-contract calls:**` row in the
  function's `CONTRACT_REFERENCE.md` entry and a comment explaining the atomicity guarantee
- [ ] `ai.md` is updated if shared types, events, or env vars changed
- [ ] `docs/CONTRACT_REFERENCE.md` is updated with new functions
- [ ] `bash scripts/check-docs.sh` passes with no failures

## Validator authorization changes

Changes to validator registration, revocation, or milestone approval logic require explicit
review from a second team member before merge — these are the trust anchors of the platform.
