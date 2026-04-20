# LoopForge

Desktop AI loop orchestrator. Tauri v2 with Rust and React. Workflow: Plan, Atomize, Execute, Monitor.

## Stack

- Backend: Rust Cargo workspace
  - `crates/ralph-core` for pure loop logic
  - `src-tauri/` for Tauri binary and adapters
- Frontend: React 19, TypeScript 5, Tailwind CSS 4, Zustand 5
- Storage: `rusqlite` for metadata and filesystem artifacts for project state
- Templates: MiniJinja `.j2`
- Package managers: Bun and Cargo

## Commands

```bash
bun run tauri dev
bun run tauri build
cargo test
bun run dev
bun run build
bun run typecheck
```

## Architecture

Three layer hexagonal architecture with explicit boundaries.

| Layer | Directory | Responsibility |
|------|------|------|
| Domain | `crates/ralph-core/src/` | Pure loop logic without UI or Tauri dependencies |
| Shell | `src-tauri/src/` | Tauri commands, events, SQLite, filesystem integration |
| UI | `src/` | React pages, components, stores, and rendering |

IPC contract: `#[tauri::command]` for request response and `app.emit()` for streaming events.

## Architecture Rationale

- Keep loop logic testable and independent from desktop runtime
- Keep persistence in Rust services through `DbState` and `rusqlite`
- Keep project artifacts as durable handoff files between wizard and runtime
- Keep UI stores as rendering state, never as canonical persisted state
- Keep atomization deterministic through MiniJinja templates

## Conventions

- No comments and no JSDoc in code
- No single character variable names
- No `eslint-disable`
- Max 200 lines per Rust and TypeScript file
- Commit format: `type(scope): description`
- Atomic commits by logical unit

## Rust Rules

- Use `thiserror` in `ralph-core` and `anyhow` in `src-tauri`
- No `unwrap()` in `ralph-core` non test code
- No `println!` in library code, use `tracing` or `log`
- New struct fields must use `#[serde(default)]`
- Child processes use `tokio::process::Command`

## React and TypeScript Rules

- Functional components only
- Zustand for app state only, not artifact source of truth
- Tailwind utility classes using design tokens from `src/tokens.css`
- Never use raw hex, rgb, or oklch color values
- Keep bindings in `src/lib/tauri.ts` for invoke and event APIs

## Testing

- Run `cargo test` after backend or core updates
- Run `bun run typecheck` after frontend TypeScript updates
- Add integration tests for new vertical slices touching storage to UI
- Keep Rust unit tests in file `#[cfg(test)]` modules
- Keep integration tests in `tests/` directories where applicable
- Validate artifact read and write behavior when changing project flow

## Safety

- Ask for confirmation before destructive git commands or release actions
- Do not commit secrets or local credential files
- Do not introduce `tauri-plugin-sql` or frontend SQL plugins
- Keep frozen modules untouched unless a planned vertical slice requires work

## File Boundaries

| Directory | Responsibility |
|------|------|
| `crates/ralph-core/src/` | Loop engine, PRD, prompt, providers, detection, verification, config |
| `src-tauri/src/` | Commands, services, storage, events, process lifecycle |
| `src-tauri/templates/` | MiniJinja templates for atomizer |
| `src/pages/` | Home, wizard flow, monitor pages |
| `src/components/` | Reusable UI components |
| `src/stores/` | Zustand stores for UI state only |
| `src/lib/` | IPC wrappers and utilities |
| `docs/` | Architecture and schema references |

## Tauri IPC Registered Command Groups

- Agents: `detect_agents`, `refresh_agents`
- Planning: `start_plan`, `write_to_plan`, `stop_plan`
- Projects: create, finalize, discard, save and resume wizard, detail and config reads
- Atomization: `run_atomizer`
- Execution: `start_loop`, `stop_loop`, `session_stats`
- Streams: `agent-output-stream`, `iteration_started`, `iteration_completed`, `story_blocked`, `rate_limit_detected`, `session_ended`
- Channels: `plan-activity`, `atomization-progress`

## Prohibitions

- Never create files above 200 lines in Rust or TypeScript
- Never use Zustand as source of truth for `plan.md`, `prd.json`, `config.json`, `draft.json`
- Never wire frozen modules into new code without planned vertical slice
- Never skip full vertical slice ordering for new features
- Never mark work complete without end to end validation
- Never use `tauri-plugin-sql`, the app uses `rusqlite` through `DbState`

## Artifact Contract

Per project directory: `~/.config/loopforge/projects/<project-id>/`

| File | Purpose | Written by | Read by |
|------|------|------|------|
| `draft.json` | Wizard snapshot | Wizard transitions | Wizard resume |
| `plan.md` | Generated plan | Plan engine | Atomizer stage 1 |
| `prd.json` | Atomic user stories | Atomizer and Atomize UI edits | Loop engine |
| `config.json` | Execution configuration | Configure step | Loop manager |
| `prompt.md` | Execution prompt | Atomizer stage 4 | Loop engine |
| `guardrails.md` | Dynamic guardrails | Atomizer and loop engine | Loop engine |

## Scoped Cursor Rules

- `.cursor/rules/rust.mdc`
- `.cursor/rules/react.mdc`
- `.cursor/rules/tauri.mdc`
- `.cursor/rules/frozen-modules.mdc`
- `.cursor/rules/vertical-slice.mdc`
- `.cursor/rules/antipatterns.mdc`
- `.cursor/rules/templates.mdc`
