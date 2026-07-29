# Validator Revocation Re-review Cascade

## Purpose

Revoking a validator stops future approvals. A revocation for misconduct also
marks that validator's prior approvals as pending re-review so scouts and
indexers can identify affected milestones without correlating every validator
record manually.

## Revocation severity

The administrator supplies a `RevocationSeverity` and reason when calling
`revoke_validator`.

| Severity | Effect on prior milestones |
|---|---|
| `Routine` | Validator is deactivated; no prior milestone flags change. |
| `ForCause` | Validator is deactivated and all indexed prior approvals are flagged. |

The severity, reason, and revocation time are retained in a `RevocationRecord`.

## Cascade and re-review

Every approval is indexed in the approving validator's history. On a for-cause
revocation, the contract iterates that history and sets a
`MilestonePendingReReview` flag for each referenced milestone. This does not
roll back player levels or delete historical milestones.

Off-chain consumers query `is_milestone_flagged(player_id, milestone_index)` to
surface the warning. An active validator may call `rereview_milestone` to clear
one pending flag after independently confirming the underlying achievement.
Both flagging and clearing emit audit events.
