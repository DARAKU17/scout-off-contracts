# Contributing

## Prerequisites

Ensure the following tools are installed at the specified minimum versions before attempting to build or test the contracts. Mismatched versions are the most common cause of opaque WASM build failures.

| Tool | Minimum version | Install / notes |
|------|----------------|-----------------|
| **Rust** (via rustup) | pinned in `rust-toolchain.toml` | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **WASM build target** | `wasm32v1-none` | `rustup target add wasm32v1-none` |
| **cargo** | ships with Rust stable | Verify: `cargo --version` |
| **clippy** | ships with Rust stable | `rustup component add clippy` |
| **rustfmt** | ships with Rust stable | `rustup component add rustfmt` |
| **stellar-cli** | **27.0.0** (pinned) | See install note below |
| **Node.js** | 20 LTS | Required only for TypeScript bindings generation — `./scripts/generate-bindings.sh` |
| **npm** | 10+ (ships with Node 20) | Required only for building/testing bindings packages |

> The repository includes `rust-toolchain.toml`, so `rustup` automatically selects the same pinned Rust version, `wasm32v1-none` target, and formatter/linter components used by CI whenever you run `cargo` or `rustup` from this directory. If a local build diverges from CI, reinstall stellar-cli at the pinned version.

### Installing the pinned stellar-cli version

`scripts/generate-bindings.sh` enforces the required `stellar-cli` version and will fail with a
clear error if the wrong version is detected. Install the exact version with:

```bash
curl -sSL https://raw.githubusercontent.com/stellar/stellar-cli/v27.0.0/install.sh | bash
```

Then verify: `stellar --version` should print `stellar 27.0.0`.

The `wasm32v1-none` target (not the older `wasm32-unknown-unknown`) is required for building Soroban contracts with `soroban-sdk 25.x`. Using the wrong target produces an ABI-incompatible WASM binary.

## Setup

```bash
rustup show
rustup component add clippy rustfmt
cp .env.example .env
```

## Before opening a PR

```bash
cargo test --workspace          # all tests must pass
cargo clippy --workspace        # zero warnings
cargo fmt --all -- --check      # formatting must be clean
```

## CI checks

The repository defines five CI jobs across `.github/workflows/ci.yml` and `.github/workflows/contract-ci.yml`. The table below lists each job, its purpose, and whether it is configured as a **required** status check (i.e., blocks merging to `main`) per GitHub's branch-protection rules.

| Job | File | What it checks | Required |
|-----|------|----------------|----------|
| `check-todos` | `ci.yml` | Scans `contracts/` for `TODO`/`FIXME`/`HACK`/`XXX` markers — fails if any are found | Yes |
| `test` | `contract-ci.yml` | Runs `cargo test --workspace`, tests `scoutchain-progress`, builds WASM release | Yes |
| `lint` | `contract-ci.yml` | Clippy (deny warnings), `rustfmt` check, shellcheck on shell scripts, docs completeness (`scripts/check-docs.sh`), bindings template validation (`scripts/check-bindings.sh`) | Yes |
| `bindings-smoke-test` | `contract-ci.yml` | Deploys all contracts to a local Soroban sandbox, generates TypeScript bindings, verifies their structure, and builds each binding package | Yes |
| `abi-export` | `contract-ci.yml` | Exports contract ABIs to `abi/*.json` using `stellar contract info interface`, validates JSON parseability, and uploads the artifacts; per `docs/VERSIONING.md` the ABI diff is how breaking changes are detected | Yes |

> **Note on the audit:** The required-status configuration above reflects the actual branch-protection rules on `main` at the time of writing. Because changing branch-protection settings requires repository admin access, any future update to the required checks must be performed by a maintainer in the repository settings (`Settings > Branches > main > Require status checks`).

### Why `abi-export` is required

Per `docs/VERSIONING.md`, the ABI export exists specifically so that reviewers can diff the output across commits to detect breaking changes. Making it a required check ensures no PR can merge without a fresh ABI artifact being generated and examined.

## Contract change checklist

- [ ] New functions have unit tests covering the happy path and at least one error case
- [ ] Any new `DataKey` variant is documented with a comment
- [ ] Cross-contract calls are documented with a comment explaining the atomicity guarantee
- [ ] `ai.md` is updated if shared types, events, or env vars changed
- [ ] `docs/CONTRACT_REFERENCE.md` is updated with new functions, events, and error codes *(enforced automatically by `scripts/check-docs.sh` in the CI lint job — the PR will fail if a `pub fn` from any `#[contractimpl]` block lacks a corresponding heading in the docs)*

### Error variant ordering

Every contract's `#[contracterror]` enum (`errors.rs`) is append-only.
**New error variants must be added at the end of the enum, never inserted
between existing variants and never renumbered.** This matches the
`docs/VERSIONING.md` policy: renumbering or removing an existing
`#[contracterror]` variant is a MAJOR breaking change because external
consumers, on-chain event listeners, and off-chain indexers key off the
numeric code.

When adding a new variant:

- Append it after the last existing variant. Do not "fill gaps" in the
  numeric sequence — a gap (e.g. `12 → 14`) is a deliberate reservation,
  not a bug, and must be preserved.
- Group related variants together by inserting a brief section comment
  above the group (e.g. `// ── Rate limiting ──`). Grouping is purely
  cosmetic for readers; numeric contiguity within a group is **not**
  required and **not** guaranteed by this convention.
- If the new variant belongs to an existing group, place it at the end
  of that group rather than at the end of the enum, so the grouping
  remains readable. This does not violate append-only because the
  variant's numeric code is the next free value after the current
  maximum — readers can still scan the file top-to-bottom to find it.
- Do not reuse a numeric code that has been removed in a prior version,
  even if the variant is long-since deprecated. On-chain history may
  still reference it.

Rationale and the full set of breaking-change rules live in
[`docs/VERSIONING.md`](VERSIONING.md).

## Validator authorization changes

Changes to validator registration, revocation, or milestone approval logic require explicit
review from a second team member before merge — these are the trust anchors of the platform.
The validator contract is covered by [`.github/CODEOWNERS`](../.github/CODEOWNERS), which
requests review from the designated validator-logic owner for changes under
`/contracts/verification/`.

Repository administrators must enable **Require review from Code Owners** in the `main`
branch-protection rule for this mapping to block merges. Before enabling that rule, confirm
that the listed owner has the required write access and update the mapping if the authorized
reviewer group changes.

## Glossary

Unfamiliar with terms like *validator*, *milestone*, *subscription tier*, or *CID*?
See [docs/GLOSSARY.md](GLOSSARY.md) for authoritative definitions of all domain-specific terms.