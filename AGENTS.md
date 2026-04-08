# LoopForge

Desktop AI loop orchestrator. Tauri v2 (Rust + React). Plan → Atomize → Execute → Monitor.

## Stack

- Backend: Rust (Cargo workspace)
  - `crates/ralph-core` — library crate: loop engine, PRD, prompt builder, providers, detection, verification
  - `src-tauri/` — Tauri v2 app binary, consumes ralph-core via Cargo path dependency
- Frontend: React 19, TypeScript 5, Tailwind CSS 4, Zustand 5
- Storage: rusqlite (SQLite, bundled) + filesystem artifacts
- Desktop: Tauri v2 (system webview, tray icon, native notifications)
- Templates: MiniJinja (.j2) for atomization pipeline
- Package manager: Bun (frontend), Cargo (Rust)

## Commands

```
bun run tauri dev       # full app development (backend + frontend)
bun run tauri build     # production build
cargo test              # Rust tests (from workspace root)
bun run dev             # frontend only (Vite dev server on :1420)
bun run build           # frontend production build
bun run typecheck       # TypeScript type check (tsc --noEmit)
```

## Architecture

Three-layer hexagonal architecture:

| Layer | Directory | Responsibility |
|-------|-----------|----------------|
| Domain | `crates/ralph-core/src/` | Pure loop logic, no UI, no Tauri deps |
| Shell | `src-tauri/src/` | Tauri commands, events, SQLite, filesystem, tray |
| UI | `src/` | React pages, Zustand stores, Tailwind styling |

IPC: `#[tauri::command]` for request/response, `app.emit()` for streaming events.

## Conventions

- No comments, no JSDoc in code
- No single-character variables
- No eslint-disable
- Max 200 lines per file
- Commit format: `type(scope): description`
- Types: feat, fix, refactor, test, chore
- Scopes: core, tauri, frontend, tokens, docs
- One atomic commit per logical unit of work

## Rust

- `thiserror` for error types in ralph-core, `anyhow` for application errors in src-tauri
- No `unwrap()` in library code (ralph-core); acceptable in tests only
- No `println!` in library code; use `tracing` or `log`
- All new struct fields use `#[serde(default)]` for backward compatibility
- Async runtime: tokio (full features)
- Child process management: `tokio::process::Command`
- Template engine: minijinja (NOT Handlebars)

## React / TypeScript

- Functional components only
- State management: Zustand (stores for app-level state only, NOT as source of truth for artifacts)
- Styling: Tailwind CSS 4 utility classes using design tokens from `src/tokens.css`
- NEVER use raw hex, rgb, or arbitrary color values
- ALWAYS use token classes: `bg-void`, `bg-surface`, `text-primary`, `border-border`, etc.
- Routing: React Router v7
- Terminal rendering: xterm.js (@xterm/xterm)
- Markdown: react-markdown + remark-gfm

## File Boundaries

| Directory | Responsibility |
|-----------|----------------|
| `crates/ralph-core/src/` | Pure loop logic: engine, PRD, prompt, providers, detection, verification, config |
| `src-tauri/src/` | Tauri commands, events, SQLite (rusqlite), filesystem artifacts, tray, agent registry |
| `src-tauri/templates/` | MiniJinja (.j2) templates for atomization pipeline |
| `src/pages/` | Page-level components: Home, wizard steps, Monitor |
| `src/components/` | Reusable React components |
| `src/stores/` | Zustand stores (app-level state only) |
| `src/lib/` | Tauri IPC bindings, utility functions |
| `docs/` | Architecture, schema, and token documentation |

## Tauri IPC — Registered Commands

Commands actually registered in `src-tauri/src/lib.rs`:

### Agents
- `detect_agents` — scan for installed CLI agents
- `refresh_agents` — re-scan on demand

### Planning
- `start_plan` — spawn agent CLI in plan mode, stream via Channel
- `write_to_plan` — forward user input to agent stdin
- `stop_plan` — terminate plan session

### Projects
- `create_project`, `finalize_draft`, `discard_draft`
- `save_wizard_state`, `resume_wizard`
- `list_projects`, `pause_project`, `resume_project`, `archive_project`
- `get_project_detail`, `get_project_stories`, `get_guardrails`, `get_project_config`
- `get_notification_prefs`, `save_notification_prefs`
- `load_existing_plan`, `load_existing_prd`

### Atomization
- `run_atomizer` — 4-stage MiniJinja pipeline

### Execution
- `start_loop`, `stop_loop`, `session_stats`

### Events (app.emit)
- `agent-output-stream` — raw agent stdout/stderr
- `iteration_started`, `iteration_completed`
- `story_blocked`, `rate_limit_detected`, `session_ended`

### Channels
- `plan-activity` — scoped to `start_plan`, carries `PlanEvent`
- `atomization-progress` — stage progress during atomization

## Prohibitions

- NEVER create files larger than 200 lines
- NEVER use Zustand as source of truth for filesystem artifacts (plan.md, prd.json, config.json)
- NEVER add `unwrap()` to ralph-core non-test code
- NEVER use `println!` in library code
- NEVER use raw color values (hex, rgb, oklch literals) — use design token classes
- NEVER wire frozen modules into new code (connections, plugins, scm_watcher, ephemeral_query, summary_generator)
- NEVER skip the vertical slice workflow (storage → command → service → event → UI)
- NEVER mark a feature as done without end-to-end testing
- NEVER use `tauri-plugin-sql` — the app uses `rusqlite` directly via `DbState`

## Artifact Contract

Per-project directory at `~/.config/loopforge/projects/<project-id>/`:

| File | Purpose | Written by | Read by |
|------|---------|-----------|---------|
| `draft.json` | Wizard snapshot (all steps) | Wizard step transitions | Wizard resume |
| `plan.md` | Generated research plan | Plan engine | Atomizer stage 1 |
| `prd.json` | Atomic user stories | Atomizer stage 4 + UI edits | Loop engine |
| `config.json` | Execution configuration | Configure step | Loop manager |
| `prompt.md` | Agent execution instructions | Atomizer stage 4 | Loop engine |
| `guardrails.md` | Dynamic guardrails | Atomizer + loop engine | Loop engine |
