# LoopForge

Desktop AI loop orchestrator. Tauri v2 with Rust backend and React frontend.

This file is a Claude session primer. See `AGENTS.md` for full canonical conventions.

## Quick Start

```fish
bun run tauri dev      # full app (backend + frontend)
cargo test             # Rust tests (workspace root)
bun run dev            # frontend only (Vite on :1420)
bun run typecheck      # TypeScript check
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
- Do not use `tauri-plugin-sql`; use backend `rusqlite` via `DbState`

## Source of Truth

| Data | Lives in | NOT in |
|------|----------|--------|
| Project metadata | SQLite (`projects`, `sessions`, `iterations`) | Zustand is not source of truth |
| Generated plan | `plan.md` on filesystem | NOT Zustand `planContent` |
| PRD / stories | `prd.json` on filesystem | NOT Zustand `stories` |
| Execution config | `config.json` on filesystem | NOT inferred from `StartLoopArgs` |
| Wizard state | `draft.json` on filesystem | NOT `wizard_state_json = "{}"` |
| Loop runtime state | Backend `LoopManagerState` | Frontend only renders snapshots |

## Safety

- Ask before running release, deploy, or destructive git operations
- Prefer read and verify first when touching contracts across frontend and backend
- Keep `AGENTS.md` as the canonical full instruction set

## Key References

- @AGENTS.md
- @docs/ARCHITECTURE.md — module boundaries, data flow
- @docs/STORY_SCHEMA.md — UserStory JSON schema
- @docs/DESIGN_TOKENS.md — OKLCH colors, fonts, token classes
- @src/tokens.css — CSS source of truth for design tokens
