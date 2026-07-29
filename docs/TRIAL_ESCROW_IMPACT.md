# Trial Escrow Economic Impact Analysis

> **This is a prioritization/impact analysis document, not a code fix.**
> Its purpose is to turn a qualitative "this seems risky" into a concrete
> number that decision-makers can act on.
>
> All projections are model-generated estimates based on the documented
> platform fee schedule. They should be revisited once real usage data exists.

---

## Background

### The confirmed code gap

The current `log_trial_offer` implementation in `contracts/scout_access/src/lib.rs`
logs a trial offer on-chain and advances a player to `EliteTier`, but the
`expire_trial_offers` function — which is supposed to sweep unconfirmed trial
offers and refund locked escrow — **is currently a no-op stub**. Nothing
releases escrow back to scouts for offers that are never confirmed.

If a `trial_offer_escrow_stroops` fee is ever introduced (the natural next step
once confirm/expire flows are built), every scout-logged trial offer that is
never confirmed or explicitly expired will permanently lock that escrow in the
contract with no recovery path under the current code.

This document cross-references:

- **expire_trial_offers implementation issue** — the code-level fix (sweep
  function that refunds stale escrow).
- **TrialEscrow enumeration-index issue** — enumerating locked escrow entries
  so admin tooling can identify and manually recover them.

---

## Fee Baseline

The documented example fee configuration from `docs/CONTRACT_REFERENCE.md`
and the `initialize` call in the README:

| Fee field | Stroops | XLM equivalent |
|-----------|---------|----------------|
| `contact_fee_stroops` | 100,000 | 0.01 XLM |
| `basic_sub_stroops` | 1,000,000 | 0.10 XLM |
| `pro_sub_stroops` | 3,000,000 | 0.30 XLM |
| `elite_sub_stroops` | 7,000,000 | 0.70 XLM |

A trial offer escrow fee is **not yet in `FeeConfig`**, but the intended design
calls for a `trial_offer_escrow_stroops` field that locks XLM from the scout
when `log_trial_offer` is called, to be released on `confirm_trial_offer` or
returned on `expire_trial_offer`. For this analysis we model two plausible
escrow values that bracket a realistic range:

| Scenario | Trial escrow per offer | Rationale |
|----------|----------------------|-----------|
| Low escrow | 1,000,000 stroops (0.1 XLM) | Nominal commitment signal |
| High escrow | 10,000,000 stroops (1.0 XLM) | Meaningful anti-spam bond |

---

## Model Assumptions

The following assumptions drive all projections. They are explicitly stated so
they can be corrected when real platform usage data exists.

| Parameter | Value | Notes |
|-----------|-------|-------|
| Monthly trial offers (ramp phase) | 50 | Early platform with limited scouts |
| Monthly trial offers (growth phase) | 500 | Platform reaching traction |
| Monthly trial offers (scale phase) | 2,000 | Regional scale |
| Never-confirmed rate — Optimistic | 10% | Most scouts and players follow through |
| Never-confirmed rate — Expected | 30% | Industry norm: significant drop-off after initial interest |
| Never-confirmed rate — Pessimistic | 60% | High churn, scouts using the platform speculatively |

A "never-confirmed" offer is one where neither `confirm_trial_offer` nor
`expire_trial_offer` is ever called — the offer is recorded on-chain and the
associated escrow is locked permanently under the current code.

**The platform is modelled as starting small (50 offers/month) and growing
linearly to 500 by month 12 and 2,000 by month 24.**

---

## Simulation: Cumulative Locked Escrow

### Locked offer count (never confirmed, cumulative)

| Horizon | Optimistic (10%) | Expected (30%) | Pessimistic (60%) |
|---------|-----------------|---------------|-------------------|
| 6 months | ~113 offers | ~338 offers | ~675 offers |
| 12 months | ~413 offers | ~1,238 offers | ~2,475 offers |
| 24 months | ~2,663 offers | ~7,988 offers | ~15,975 offers |

> **Derivation**: monthly offers grow linearly from 50 to 500 over months 1–12
> and 500 to 2,000 over months 13–24. The never-confirmed fraction accumulates
> each month.

### Locked XLM at low escrow (0.1 XLM per offer)

| Horizon | Optimistic | Expected | Pessimistic |
|---------|-----------|---------|------------|
| 6 months | **~11 XLM** | **~34 XLM** | **~68 XLM** |
| 12 months | **~41 XLM** | **~124 XLM** | **~248 XLM** |
| 24 months | **~266 XLM** | **~799 XLM** | **~1,598 XLM** |

### Locked XLM at high escrow (1.0 XLM per offer)

| Horizon | Optimistic | Expected | Pessimistic |
|---------|-----------|---------|------------|
| 6 months | **~113 XLM** | **~338 XLM** | **~675 XLM** |
| 12 months | **~413 XLM** | **~1,238 XLM** | **~2,475 XLM** |
| 24 months | **~2,663 XLM** | **~7,988 XLM** | **~15,975 XLM** |

> At time of writing, XLM trades between $0.09–$0.12 USD. At $0.10 USD/XLM,
> the pessimistic 24-month high-escrow scenario represents **~$1,598 USD** in
> permanently locked capital. At a higher price ($0.30 USD/XLM) this becomes
> **~$4,793 USD**.
>
> These are not large absolute numbers for a funded protocol, but they grow
> monotonically with no recovery path and represent capital scouts have paid
> that they cannot recover — a trust and user-experience problem that
> compounds over time.

---

## Sensitivity to Escrow Value

The escrow fee is the most impactful variable. The table below shows locked
XLM at 24 months under the Expected (30%) never-confirmed rate for a range
of potential escrow values:

| Escrow per offer | Locked XLM @ 24 months (expected) |
|-----------------|-----------------------------------|
| 500,000 stroops (0.05 XLM) | ~399 XLM |
| 1,000,000 stroops (0.10 XLM) | ~799 XLM |
| 5,000,000 stroops (0.50 XLM) | ~3,994 XLM |
| 10,000,000 stroops (1.00 XLM) | ~7,988 XLM |
| 50,000,000 stroops (5.00 XLM) | ~39,938 XLM |

Even a modest 0.10 XLM escrow produces nearly 800 XLM of permanently locked
capital over 24 months at a 30% never-confirmed rate. A more meaningful
1 XLM bond approaches 8,000 XLM.

---

## Recommendation

### 1. Prioritize the `expire_trial_offers` fix — YES

This analysis justifies **prioritizing the `expire_trial_offers` fix ahead of
new features** once `trial_offer_escrow_stroops` is introduced. The locked
capital is not recoverable without either a contract upgrade or an admin
escape hatch, and it grows without bound. The 24-month expected-scenario
number (800–8,000 XLM depending on escrow amount) is large enough to affect
scout trust and platform credibility.

The fix is also low-risk: `expire_trial_offers` is already stubbed; it needs
a well-defined expiry window, an index over unconfirmed offers, and a token
transfer back to the scout. This is a bounded, testable change.

### 2. Implement an interim admin mitigation — YES, in parallel

While the proper sweep function is being built, add an admin-callable
`admin_refund_trial_escrow(player_id: u64, offer_index: u32, to: Address)`
function — analogous to the existing `refund_subscription` — that allows
manual rescue of identified stuck escrow entries.

This provides a zero-downtime safety valve: operations can manually refund
scouts who complain about locked capital while the automated fix is
developed and audited.

The TrialEscrow enumeration-index fix (tracked separately) is a prerequisite
for making this admin tooling practical, since without an index, identifying
stuck offers requires iterating all `(player_id, offer_index)` pairs off-chain.

### 3. Do not introduce `trial_offer_escrow_stroops` without the fix in place

The current implementation logs trial offers without collecting any escrow,
so no capital is currently at risk. **Do not add the escrow collection step**
to `log_trial_offer` until `expire_trial_offers` is functional and the
enumeration index exists. Introducing escrow collection before the release
path exists converts a future risk into an immediate certainty.

---

## Caveats and Invitation to Correct

- All volume projections are illustrative. Real platform growth may be faster,
  slower, or non-linear.
- The never-confirmed rate is the most uncertain variable. A platform with
  strong notifications, a mobile app, and engaged scouts may achieve < 10%.
  A platform used primarily by exploratory scouts may exceed 60%.
- XLM price volatility means USD impact estimates could differ substantially
  from the values shown here.
- The model does not account for scouts who abandon wallets entirely (their
  escrow is locked regardless of the expire mechanism unless admin can recover
  it manually).

**Once real platform data is available, replace the assumption table above
with measured values and re-run the projections.**

---

## Related Issues

| Issue | Description |
|-------|-------------|
| `expire_trial_offers` implementation | Code-level fix: implement the sweep function that refunds unconfirmed trial escrow after a configurable expiry window |
| TrialEscrow enumeration-index | Build an index that makes stuck-escrow enumeration practical for both the sweep function and admin tooling |
