# Dispute Resolution and Jury Escalation

> **Status: Implemented** — shipped in `feature/dispute-jury-escalation-1036` (issue #1036).
> See `docs/CONTRACT_REFERENCE.md` for the complete function reference and
> `docs/DISPUTE_JURY.md` for the design specification.

## Scope

Milestone disputes use an impact score supplied when the dispute is filed. The
contract routes low-impact disputes to the existing administrator path and
escalates high-impact disputes to independent validator voting.

## Configuration

The administrator sets `impact_threshold`, `quorum`, and `voting_window_secs`
with `set_jury_config`. Defaults are a threshold of 100, a quorum of 3, and a
seven-day voting window. A dispute snapshots its quorum and deadline when it
is filed, so later configuration changes cannot alter an in-progress vote.

| Impact score | Resolution path |
|---|---|
| Below threshold | Admin calls `resolve_dispute` |
| At or above threshold | Validator jury calls `tally_dispute` |

## Jury rules

- Any registered, active validator may vote once before the deadline.
- The validator who approved the disputed milestone is conflicted and cannot vote.
- A non-tied result may be finalized as soon as the quorum is reached.
- Once the deadline passes, anyone may tally the vote. A tie or a vote below
  quorum rejects the dispute, leaving the original milestone in place.
- An upheld result records the dispute outcome only; it does not roll back
  player progress automatically.

## Audit trail

Each dispute stores its filing details, route, deadline, quorum, vote totals,
and final result. Individual votes are stored by validator. The contract emits
`milestone_disputed`, `dispute_vote_cast`, `dispute_resolved`, and
`dispute_tallied` events for indexers and reviewers.
