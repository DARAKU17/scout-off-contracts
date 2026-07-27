# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| `main` (testnet) | ✅ Active development |
| Older branches | ❌ Not supported |

## Reporting a Vulnerability

### Primary channel — GitHub Private Vulnerability Reporting (recommended)

Use [GitHub's built-in Private Vulnerability Reporting](https://github.com/scout-off/scout-off-contracts/security/advisories/new)
to submit a vulnerability report. This channel is **live and monitored** and is the
preferred path for all responsible disclosures.

### Secondary channel — email

> ⚠️ **PLACEHOLDER — NOT YET OPERATIONAL**
>
> `security@scout-off.io` is listed below as a secondary contact address, but it is
> **not yet a live, monitored inbox**. It **must not** be relied upon as a real
> reporting channel until a team member has confirmed the inbox is active and
> monitored.
>
> **Before this responsible-disclosure process is used for any real vulnerability
> report, the following action must be completed:**
>
> - [ ] Stand up and verify `security@scout-off.io` as a monitored mailbox, then
>       remove this warning block and update the status below.
>
> Tracked in: [scout-off/scout-off-contracts #879](https://github.com/scout-off/scout-off-contracts/issues/879)

Email: `security@scout-off.io` *(placeholder — monitored when operational)*

Until `security@scout-off.io` is confirmed operational, please use **GitHub Private
Vulnerability Reporting exclusively** for all security disclosures.

## Response Timeline

| Milestone | Target |
|-----------|--------|
| Acknowledgement | Within 48 hours (GitHub PVR only while email is not live) |
| Initial assessment | Within 7 days |
| Patch / mitigation | Depends on severity; critical issues prioritised |

## Scope

The following are in scope:

- All Soroban smart contracts under `contracts/`
- Deployment and initialization scripts under `scripts/`
- TypeScript binding packages under `bindings/`

Out of scope:

- Third-party dependencies (report directly to the upstream maintainer)
- Theoretical vulnerabilities with no practical exploit path

## Disclosure Policy

We follow coordinated responsible disclosure. Please do not publicly disclose a
vulnerability until a patch has been released or we have agreed on a disclosure
timeline together.
