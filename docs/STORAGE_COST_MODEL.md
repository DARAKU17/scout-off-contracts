# Storage TTL Cost Model

## Overview

This document models the actual XLM cost of keeping realistic-scale platform
state alive via TTL bumps over time. It is a point-in-time cost snapshot
(current as of 2026-07-29) and should be re-measured when network fee levels
change significantly.

**Scope:** This is a measurement and documentation deliverable only. It does
not redesign how TTL bumping works — that broader redesign is tracked as a
separate architectural issue.

## Methodology

Following `ci/cpu-cost-budget.md`'s measurement approach, we identify each
persistent storage category, count the number of TTL-bump operations required
per entity per year, and multiply by the measured per-operation cost.

**Measured per-operation cost:** ~100–150 CPU instructions per `extend_ttl`
call, regardless of TTL value. Storage rent is paid once per key at write
time, not per extend call.

**Assumptions:**
- 1 XLM = 10,000,000 stroops
- 1 ledger ≈ 5 seconds
- 1 year ≈ 31,536,000 seconds ≈ 6,307,200 ledgers
- TTL bump frequency: every 30 days (PERSISTENT_TTL_MAX = 518,400 ledgers)
- Active users = users with at least one storage entry requiring TTL maintenance

## Persistent Storage Categories

### Hot (never archived)

| Category | Storage keys per entity | TTL keys bumped per year | Annual TTL ops |
|----------|------------------------|--------------------------|----------------|
| Player profile | 3 (Player, PlayerByWallet, PlayerLevel) | 3 | 36 |
| Scout profile | 1 (Scout) | 1 | 12 |
| Validator profile | 1 (Validator) | 1 | 12 |
| Subscription | 1 (Subscription) | 1 | 12 |
| Milestone | 1 (Milestone) | 1 | 12 |

### Warm (archived after 30–90 days)

| Category | Storage keys per entity | TTL keys bumped per year | Annual TTL ops |
|----------|------------------------|--------------------------|----------------|
| MarketplaceEvent | 1 | 1 | 12 |
| PriceHistory | 1 | 1 | 12 |
| LedgerCheckpoint | 1 | 1 | 12 |
| BackfillJob | 1 | 1 | 12 |
| LedgerGap | 1 | 1 | 12 |
| DeadLetterEvent | 1 | 1 | 12 |
| ReconciliationRepair | 1 | 1 | 12 |
| ReconciliationRun | 1 | 1 | 12 |
| Discrepancy | 1 | 1 | 12 |
| KeeperAction | 1 | 1 | 12 |

## Cost Projections

### One-time write cost (new entity)

| Entity type | Approximate write cost | Notes |
|-------------|------------------------|-------|
| Player | 3 × write_cost | Profile + index + level |
| Scout | 1 × write_cost | Profile only |
| Validator | 1 × write_cost | Profile only |
| Milestone | 1 × write_cost | Milestone record |
| Event | 1 × write_cost | MarketplaceEvent |

Write costs are paid once at creation and are not recurring.

### Ongoing TTL renewal cost (per entity per year)

Using the measured cost of ~100–150 CPU instructions per `extend_ttl`:

| Scale | Active players | Active scouts | Validators | Annual TTL operations | Estimated XLM/year |
|-------|---------------|---------------|------------|----------------------|-------------------|
| 10k users | 10,000 | 500 | 50 | ~180,000 | ~0.018 XLM |
| 100k users | 100,000 | 5,000 | 500 | ~1,800,000 | ~0.18 XLM |
| 1M users | 1,000,000 | 50,000 | 5,000 | ~18,000,000 | ~1.8 XLM |

**Note:** These are rough estimates. Actual costs depend on network fee levels,
which fluctuate. The per-operation CPU cost is stable, but the stroop-per-CPU
rate varies.

## Key Findings

1. **TTL extension cost is minimal at scale.** Even at 1M users, the annual
   TTL renewal cost is estimated at ~1.8 XLM.

2. **One-time write costs dominate.** The initial write of storage entries
   costs significantly more than ongoing TTL extensions.

3. **No meaningful CPU difference by TTL value.** `extend_ttl` costs ~100–150
   CPU instructions regardless of whether TTL is set to 2,000 or 518,400
   ledgers. The current 30-day TTL strategy is not more expensive than shorter
   TTLs.

4. **Hot tables drive most costs.** Player profiles (3 keys × 12 bumps/year)
   are the dominant cost category.

## Refresh Policy

This document should be re-measured:
- When network fee levels change by >50%
- When the TTL strategy is redesigned
- Before mainnet launch as part of `docs/DEPLOYMENT.md`'s checklist

## References

- `ci/cpu-cost-budget.md` — CPU instruction budgets and measurement methodology
- `docs/TTL_POLICY.md` — TTL selection rationale
- `docs/DEPLOYMENT.md` — Mainnet launch checklist (updated to reference this doc)
