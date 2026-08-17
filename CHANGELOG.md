# Changelog

This file records notable versioned changes to the ScoutOff contracts. The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the repository versioning policy lives in [docs/VERSIONING.md](docs/VERSIONING.md).

## Entry conventions

Future entries should use the following structure:

- Version: `vMAJOR.MINOR.PATCH`
- Release date: `YYYY-MM-DD`
- Contracts affected: the contract or contracts changed by the release
- Summary: a short description of the externally observable change
- Classification: `Breaking (MAJOR)` or `Non-breaking (MINOR)`

Entries must be kept in reverse chronological order. Any pull request that requires a MINOR or MAJOR version bump must add or update the corresponding changelog entry.

The initial v0.1.0 entry below retains the year-only date because no exact historical release date is available. Adoption of this changelog does not require retroactive entries for earlier unversioned changes.

## Unreleased

Use the structure below for upcoming MINOR or MAJOR contract changes:

- Version: `vX.Y.Z`
- Release date: `YYYY-MM-DD`
- Contracts affected: `progress`, `registration`, `scout_access`, `verification` (or a subset)
- Summary: a concise description of the externally observable change
- Classification: `Breaking (MAJOR)` or `Non-breaking (MINOR)`

> **Breaking-change classification rules:** See [docs/VERSIONING.md — What Constitutes a Breaking Change](VERSIONING.md#what-constitutes-a-breaking-change) for the full criteria (storage layout changes, function signature changes, error code renumbering, event schema changes, cross-contract interface changes).

- Version: `v0.2.0 (verification)`
- Release date: `2026-07-29`
- Contracts affected: `verification`
- Summary: Added `attest_milestone`, an on-chain k-of-n threshold consensus scheme for milestone approval. `attest_milestone(validator_wallet, player_id, description, evidence_hash)` records one independent, asynchronous vote per call; once `threshold` distinct, currently-active validators have voted for the same `(player_id, evidence_hash)` claim within a configurable voting window, the milestone commits and `progress.advance_level` is cross-called exactly once. Also added `set_milestone_threshold`/`get_milestone_threshold`, `set_voting_window_secs`/`get_voting_window_secs`, `get_pending_claim`, `has_attested`, and `is_attestation_window_expired`; three error codes appended (`DuplicateAttestation` 26, `TooManyPendingVotes` 27, `ThresholdModeRequiresAttestation` 28); `revoke_validator`/`batch_revoke_validators` now retroactively strip a revoked validator's still-open vote from any pending claim's tally. `approve_milestone`'s signature and default behavior (`threshold = 1`) are unchanged for existing callers; it is only gated (`ThresholdModeRequiresAttestation`) once an operator opts in via `set_milestone_threshold(n >= 2)`. A follow-up audit closed a gap where `submit_attested_milestone` (the off-chain ed25519-relay commit path) did not check the same threshold gate and remained a single-signature bypass of k-of-n mode; also fixed `has_attested` returning a stale `true` for votes past an unrolled-over expired window, and a `MAX_PENDING_VOTES_PER_VALIDATOR` bookkeeping bug that double-counted a validator's own claim when their revote was what triggered that claim's lazy round-bump.
- Classification: `Non-breaking (MINOR)`

- Version: `v0.2.0`
- Release date: `2026-07-28`
- Contracts affected: `scout_access`
- Summary: `batch_contact_players` now returns `ProContactLimitReached` (20) instead of `ContactQuotaExceeded` (18) when the Pro-tier monthly contact limit is exceeded. Error code 18 is reserved/deprecated. `check_pro_contact_quota_with_count` unified with `pay_to_contact`'s inline quota check on the same error code.
- Classification: `Breaking (MAJOR)`

> **Migration guide:** Clients that previously matched `ContactQuotaExceeded` (18) from `batch_contact_players` must update to `ProContactLimitReached` (20). Both `pay_to_contact` and `batch_contact_players` now return the same error code for equivalent quota-exceeded states.

## v0.1.0 - 2025

- Version: `v0.1.0`
- Release date: `2025`
- Contracts affected: `progress`, `registration`, `scout_access`, `verification`
- Summary: Initial release — all four contracts with full test coverage
  - Baseline includes milestone disputes, batch contact operations, escrow-backed
    trial offers, and Pro-tier contact quotas; these were part of v0.1.0
    rather than later unversioned additions.
- Classification: `Non-breaking (initial release baseline)`

This entry is treated as the baseline for the initial public release rather than a change from an earlier public version.
