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

## v0.1.0 - 2025

- Version: `v0.1.0`
- Release date: `2025`
- Contracts affected: `progress`, `registration`, `scout_access`, `verification`
- Summary: Initial release — all four contracts with full test coverage
- Classification: `Non-breaking (initial release baseline)`

This entry is treated as the baseline for the initial public release rather than a change from an earlier public version.
