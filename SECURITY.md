# Security Policy

## Supported Versions

LoopForge is under active development. Only the latest release on the `main` branch receives security fixes.

| Version | Supported |
| ------- | :-------: |
| `main`  | yes       |
| `< v0.1.0` | no     |

## Reporting a Vulnerability

Please do not open a public issue for security problems. Report them privately so the fix can ship before the exploit spreads.

1. Preferred channel is GitHub private vulnerability reporting. Open a report through the `Security` tab of this repository once the feature is enabled.
2. Alternative channel: send a short description, reproduction steps, and impact analysis to the maintainers through a direct message on GitHub.
3. You will receive an acknowledgment within 72 hours. A remediation plan is shared within 7 days once the issue is validated.

Please avoid testing vulnerabilities against production deployments that you do not own. Never run exploits against third party services through LoopForge.

## Scope

The following areas receive priority:

- Local execution of shell commands by the Ralph loop engine.
- IPC boundary between the Tauri backend and the frontend renderer.
- Storage of Ralph credentials, environment variables, and project metadata in SQLite and the operating system keychain.
- Supply chain integrity of Rust crates, npm packages, and GitHub Actions referenced in workflows.
- Agent providers that execute generated code on the developer machine.

Out of scope:

- Social engineering of maintainers or users.
- Attacks that require physical access to the developer machine.
- Vulnerabilities already tracked in public advisories for upstream dependencies when a fix is pending upstream.

## Handling and disclosure

- Fixes are developed on a private branch or a security advisory draft.
- A coordinated release with release notes is published once the patch lands on `main`.
- CVEs are requested through GitHub when the impact warrants it.
- Reporters are credited in the advisory unless they request anonymity.
