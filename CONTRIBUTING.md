# Contributing to LoopForge

Thank you for taking the time to contribute. LoopForge is a focused desktop product with strict conventions so that agent driven iteration on the codebase stays predictable. Please read this document before opening a pull request.

## Before you start

1. Read [AGENTS.md](AGENTS.md). It is the canonical spec for layer boundaries, the artifact contract and the conventions every contribution follows.
2. Skim [CLAUDE.md](CLAUDE.md) for a shorter Claude oriented primer.
3. Review the scoped rules in `.cursor/rules/` that apply to the area you plan to touch (`rust.mdc`, `react.mdc`, `tauri.mdc`, `vertical-slice.mdc`, `templates.mdc`, `antipatterns.mdc`, `frozen-modules.mdc`).

## Development setup

```bash
git clone https://github.com/taberoajorge/loopforge.git
cd loopforge
bun install
bun run tauri dev
```

Prerequisites: Rust 1.77 or newer, Bun 1.3 or newer, and the platform toolchain required by Tauri. See the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/).

## Conventions

- Max 200 lines per Rust or TypeScript file.
- No comments, no JSDoc, no single character variables, no `eslint-disable`.
- Use design tokens from `src/tokens.css` for colour. Never use raw hex, rgb or oklch values in components.
- Use `thiserror` in `crates/ralph-core` and `anyhow` in `src-tauri`.
- Child processes always go through `tokio::process::Command`.
- New struct fields must use `#[serde(default)]`.
- Frontend state lives in Zustand stores. The filesystem and SQLite remain the source of truth for `plan.md`, `prd.json`, `config.json` and `draft.json`.

## Commit format

```
type(scope): description
```

Where `type` is one of `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `style`, `perf`, `ci` and `scope` is the touched module.

Example: `feat(loop-engine): propagate rate limit notices to the monitor`.

Commits are atomic by logical unit. If a change touches the loop engine and the UI, split it into two commits.

## Testing gate

Run the relevant checks before opening a pull request:

```bash
cargo test --workspace
bun run typecheck
bun run test
bun run lint
```

For user facing changes also run at least one smoke e2e:

```bash
bun run e2e:smoke
```

## Opening a pull request

1. Branch off `main` with a descriptive name (`feat/monitor-cost-tab`, `fix/atomizer-json-parse`).
2. Keep the PR focused. Unrelated refactors belong in a separate PR.
3. Fill in the PR description with what changed, why it changed, and how you verified it.
4. Link any related issue or discussion.
5. Wait for CI. Green is required to merge.

## Proposing larger changes

For anything that touches a layer boundary, adds a new crate, changes the artifact contract or introduces a new provider, open an issue labelled `proposal` first. Describe the motivation, the proposed vertical slice and the testing plan. A short design exchange there saves a long review cycle later.

## Reporting bugs

- Include your operating system, LoopForge version, agent CLI name and version, and the relevant slice of `activity.log` and `error.log`.
- Redact anything that looks like an API key or an email address.
- If the bug is reproducible, include the minimal PRD that triggers it.

## Security disclosures

Do not open a public issue for security problems. Email the maintainer listed in `AGENTS.md` with the details.
