# Security Policy

## Reporting a Vulnerability

We take the security of ScoutChain and its smart contracts seriously. If you believe you have discovered a security vulnerability, please report it to us privately.

### Private Reporting Channel

**Please report vulnerabilities via GitHub Private Vulnerability Reporting:**

1. Go to the [scout-off/scout-off-contracts](https://github.com/scout-off/scout-off-contracts) repository
2. Navigate to **Settings** → **Security** → **Private vulnerability reporting** (or use the "Report a vulnerability" link under the repository's Security tab)
3. Submit a detailed report describing the vulnerability, including:
   - The affected contract(s) and function(s)
   - Steps to reproduce the issue
   - Potential impact and exploit scenario
   - Any suggested remediation (if known)

**Alternative contact:** `security@scout-off.io` (placeholder — monitored when operational)

**Please do not** open public GitHub issues, Discord threads, or support tickets for security vulnerabilities.

---

## Response Commitments

We aim to acknowledge receipt of vulnerability reports within the following timeframes:

| Severity | Initial Acknowledgment | Target Remediation Timeline |
|----------|----------------------|---------------------------|
| **Critical** | Within 24 hours | Emergency patch as soon as possible |
| **High** | Within 48 hours | Patch within 7 days |
| **Medium** | Within 5 days | Patch within 30 days |
| **Low** | Within 14 days | Patch within 90 days or next release |

*Timeframes are measured from the initial report submission and assume sufficient information to reproduce and validate the issue.*

---

## Scope

The following smart contracts are **in scope** for security reports:

| Contract | Purpose |
|----------|---------|
| `registration` | Player & scout on-chain identity management |
| `verification` | Validator registry & milestone approvals |
| `progress` | Four-tier progress level state machine |
| `scout_access` | Subscriptions, pay-to-contact, trial offers |

Supporting infrastructure (bindings, scripts, configuration, documentation) is **out of scope** unless a vulnerability in those components directly impacts contract security.

---

## Emergency Response: Immediate Mitigation

If you are reporting an **actively exploited critical vulnerability**, refer to the **[Emergency: Pause All Contracts](docs/RUNBOOK.md#emergency-pause-all-contracts)** procedure in the runbook for immediate mitigation steps.

The platform admin can:
1. Run `./scripts/emergency-pause.sh` to halt all state-changing contract operations
2. Verify all four contracts are paused via `health()` queries
3. Coordinate with the reporting researcher on root-cause analysis and remediation

See [`docs/RUNBOOK.md`](docs/RUNBOOK.md) for the full emergency-pause procedure.

For admin key-loss incidents specifically, also see
[`docs/RUNBOOK.md#emergency-admin-key-loss--compromise`](docs/RUNBOOK.md#emergency-admin-key-loss--compromise).

---

## Responsible Disclosure Policy

We ask that security researchers:

1. **Report privately first** — Give us a reasonable opportunity to investigate and remediate before any public disclosure
2. **Provide sufficient detail** — Include reproduction steps, affected versions, and proof-of-concept where possible
3. **Act in good faith** — Avoid actions that degrade platform availability, compromise user data, or access production systems beyond what is necessary for proof-of-concept validation
4. **Allow reasonable time for remediation** — Respect the target timelines above before disclosing to third parties

We commit to:

- Acknowledging receipt of your report within the target timeframes above
- Investigating and validating reported issues promptly
- Keeping you informed of remediation progress
- Giving credit for valid, previously unreported vulnerabilities (if desired) in release notes and security advisories

---

## Future Bug Bounty Program

A formal bug bounty program is under evaluation. This policy will be updated when the program launches. In the meantime, we greatly appreciate responsible disclosure and will publicly acknowledge valid reports.

---

## Policy Maintenance

This security policy is reviewed and updated as the platform evolves. Significant changes will be communicated via the repository's release notes and changelog.
