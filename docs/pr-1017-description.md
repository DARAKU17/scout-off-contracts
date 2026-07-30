# Fix `filter_players` Pagination Cursor Mismatch

## Problem

`registration.filter_players`'s documented pagination contract was internally inconsistent. The doc comment instructed callers to pass the previously-returned `next_cursor` value back as `offset`, but `next_cursor` was set to a raw `player_id` while `offset` was compared against a running count of eligible entries (`skipped`). These are different units that only coincide in the simplest case (contiguous player IDs, no position filter, no deactivated players before the page boundary).

**Concrete failure scenario (from the issue):**
Register 5 Forwards (IDs 1–5), 1 Midfielder (ID 6), then 3 more Forwards (IDs 7–9).
- `filter_players(position="Forward", offset=0, limit=4)` → returns `[1,2,3,4]`, `next_cursor=5`
- `filter_players(position="Forward", offset=5, limit=4)` → old code skipped 5 eligible entries `[1,2,3,4,5]` instead of 4, losing player 5

This happens because `offset=5` is compared as a count of eligible entries, but player 5 (a Forward) was the page-boundary trigger — its `player_id` was returned as `next_cursor`, yet passing it back as an offset caused one too many eligible entries to be skipped.

## Root Cause

In both the region-filtered fast path and the full-scan slow path (`contracts/registration/src/lib.rs:1292`, `:1330`):

```rust
// BEFORE: next_cursor = player_id;  ← wrong unit
next_cursor = (skipped + results.len()) as u64;  // AFTER: count-based
```

`offset` is a count of eligible entries to skip (`skipped < offset`), but `next_cursor` was the raw `player_id`. The doc comment in `FilterResult` (`types.rs:112`) even referred to the parameter as "cursor" instead of "offset."

## Fix

### 1. Count-based `next_cursor` (both paths)

Changed both `next_cursor = player_id` assignments to `next_cursor = (skipped + results.len()) as u64`. This makes `next_cursor` the count of eligible entries processed so far — the same unit as `offset` — so passing it back correctly resumes from where the previous page ended.

`next_cursor = 0` remains the sentinel for "no further results" (it only stays `0` when the loop completes without hitting the page limit).

### 2. Doc comments reconciled

- **`FilterResult.next_cursor`** (`types.rs:112`): `"cursor"` → `"offset"`
- **`filter_players`** (`lib.rs:1223–1227`): Added clarification that `offset` is a count, not a player ID

### 3. `CONTRACT_REFERENCE.md` updated

Function signature now includes `offset: u32, limit: u32` and return type `FilterResult`. Pagination contract is documented.

### 4. Tests

- **`test_filter_players_pagination`**: Now uses `page1.next_cursor` as the offset for page 2 (tests the documented contract)
- **`test_filter_players_pagination_cursor_no_gaps`** (new): Full pagination walk with a position filter gap at the page boundary. Asserts all 8 Forwards are returned exactly once with no gaps or duplicates
- **Gas-griefing tests**: Fixed `result.players` → `result.profiles` (compile error, field name mismatch)

closes #1017
