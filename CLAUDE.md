# LoopForge

Desktop AI loop orchestrator. Tauri v2 (Rust backend) + React 19 frontend. Three-phase workflow: Plan → Atomize → Execute with real-time Monitor.

**Status**: Reboot in progress. Core flow works structurally but has broken contracts between layers. See "Known Broken Contracts" below.

## Quick Start

```fish
bun run tauri dev      # full app (backend + frontend)
cargo test             # Rust tests (workspace root)
bun run dev            # frontend only (Vite on :1420)
bun run typecheck      # TypeScript check
```

## Architecture

Three-layer hexagonal: domain → shell → presentation.

```
ralph-core (library, zero Tauri deps)
  └── loop_engine, prd, prompt, providers, detection, verification, config
src-tauri (Tauri v2 binary, consumes ralph-core)
  └── 16 modules in src-tauri/src/ (flat, being restructured to commands/services/storage)
src/ (React 19 + TypeScript + Tailwind CSS 4 + Zustand)
  └── pages/ (Home, wizard/[Describe,Plan,Atomize,Configure,Launch], monitor/Monitor)
```

## Critical Rules

- Max 200 lines per file (Rust and TypeScript)
- No comments, no JSDoc
- No single-character variables
- No eslint-disable
- No raw color values — design tokens only (`bg-void`, `text-primary`, `border-border`)
- `thiserror` for Rust error types, `anyhow` for application errors
- No `unwrap()` in library code (ralph-core); acceptable in tests only
- No `println!` in library code; use `tracing` or `log`
- All new struct fields use `#[serde(default)]`
- Atomic commits: `type(scope): description`

## Source of Truth

| Data | Lives in | NOT in |
|------|----------|--------|
| Project metadata | SQLite (`projects`, `sessions`, `iterations`) | Zustand is not source of truth |
| Generated plan | `plan.md` on filesystem | NOT Zustand `planContent` |
| PRD / stories | `prd.json` on filesystem | NOT Zustand `stories` |
| Execution config | `config.json` on filesystem | NOT inferred from `StartLoopArgs` |
| Wizard state | `draft.json` on filesystem | NOT `wizard_state_json = "{}"` |
| Loop runtime state | Backend `LoopManagerState` | Frontend only renders snapshots |

## Known Broken Contracts (being fixed)

1. **Plan edits don't persist** — `Plan.tsx` edits stay in local state, atomizer reads `plan.md` from disk
2. **PRD edits don't reach runtime** — Zustand edits never saved to `prd.json`, loop runs stale version
3. **Configure is decorative** — UI captures 9 fields, `StartLoopArgs` only carries 4, `build_ralph_config` only applies `max_iterations`
4. **Wizard persistence is nominal** — saves `"{}"` on exit, rehidration ignores `wizard_state_json`
5. **Plan cleanup is a stub** — `cleanup_session_by_id` is empty, child processes leak
6. **V2 features are facades** — ephemeral_query returns "future update", plugin_registry is static, scm_watcher not integrated
7. **Summary uses wrong directory** — git diff runs against artifacts dir, not working directory

## Antipatterns

BAD: Edit stories in Zustand without persisting → loop runs stale PRD
GOOD: Persist edits to `prd.json` via IPC command, UI reads back from backend

BAD: Store wizard state as `"{}"` in SQLite
GOOD: Write `draft.json` with full wizard snapshot to filesystem

BAD: `StartLoopArgs` with 4 fields when Configure captures 9
GOOD: `config.json` as mandatory artifact before launch, all fields flow to `build_ralph_config`

BAD: `fn cleanup_session_by_id(_id: &str) {}`
GOOD: Kill child process, remove session from state, flush partial `plan.md`

BAD: Create 800-line module with mixed concerns
GOOD: Split into commands/ (thin handlers), services/ (logic), storage/ (data access)

## Frozen Modules (DO NOT TOUCH)

These modules exist but are not connected end-to-end. Do not modify, extend, or wire them into new code until their vertical slice is planned:

- `connections.rs` + `Connections.tsx` — multi-repo workspace
- `plugin_registry.rs` + `Plugins.tsx` — plugin system
- `scm_watcher.rs` — review comment routing
- `ephemeral_query.rs` + `EphemeralOverlay.tsx` — query during execution
- `summary_generator.rs` — proof-of-work reports

## Vertical Slice Workflow

When adding a new feature, implement in this order:

1. **Storage** — SQLite migration or filesystem artifact schema
2. **Models** — Rust structs with `Serialize`/`Deserialize`
3. **Service** — Business logic in `src-tauri/src/`
4. **Command** — `#[tauri::command]` thin wrapper
5. **Event** — Add to typed event catalog if streaming is needed
6. **IPC binding** — TypeScript invoke wrapper in `src/lib/tauri.ts`
7. **UI** — React component consuming the IPC binding
8. **Test** — Integration test proving the full slice works

Never ship a step without the ones before it. No "scaffolding" without wiring.

## Artifact Contract

Per-project directory at `~/.config/loopforge/projects/<project-id>/`:

| File | Written by | Read by |
|------|-----------|---------|
| `draft.json` | Wizard (each step transition) | Wizard (resume on reopen) |
| `plan.md` | Plan engine (streaming + flush) | Atomizer (stage 1 input) |
| `prd.json` | Atomizer (stage 4 output), Atomize UI (edits) | Loop engine (story iteration) |
| `config.json` | Configure step | Loop manager (`build_ralph_config`) |
| `prompt.md` | Atomizer (stage 4) | Loop engine (prompt builder) |
| `guardrails.md` | Atomizer (stage 4), loop engine (appends) | Loop engine (context) |

## Key References

- @docs/ARCHITECTURE.md — module boundaries, data flow
- @docs/STORY_SCHEMA.md — UserStory JSON schema
- @docs/DESIGN_TOKENS.md — OKLCH colors, fonts, token classes
- @src/tokens.css — CSS source of truth for design tokens
