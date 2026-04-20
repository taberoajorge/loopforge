# Backend Decoupling Master Plan

## Overview

Transform LoopForge's backend into a **frontend-agnostic service layer** using **Hexagonal Architecture (Ports and Adapters)**. The backend will handle all business logic and expose a stable contract consumable by any frontend: React/Tauri, CLI, REST API, or third-party clients.

**Architecture Pattern**: Based on the [Yaak project refactor](https://github.com/mountain-loop/yaak/pull/354) and standard Tauri Core-Shell patterns, which successfully decoupled core logic from Tauri to enable CLI, server, MCP, and TUI interfaces.

---

## Current State Summary

### Already Decoupled ✓

| Component | Location | Notes |
|-----------|----------|-------|
| Domain logic | `crates/ralph-core` | Zero Tauri deps, pure loop engine |
| Domain ports | `ralph-core/src/ports.rs` | GitOps, HealthChecker, GuardrailStore, StateStore, ActivityLogger |
| Service traits | `crates/loopforge-app-core` | ProjectService, SessionService, MonitorService |
| DTOs | `crates/app-services` | SessionStats, MonitorEvent with `#[serde(default)]` |
| Tauri path adapter | `src-tauri/src/adapters/paths_tauri.rs` | TauriPathResolver (started) |

### Coupled to Tauri ✗

| Component | Location | Coupling Type |
|-----------|----------|---------------|
| Commands | `src-tauri/src/commands/*.rs` | `tauri::State<T>`, `AppHandle` |
| Database | `src-tauri/src/storage/db.rs` | `DbState` wraps Tauri lifecycle |
| Events | `src-tauri/src/loop_manager/*.rs` | `app.emit()` direct calls |
| Paths | `src-tauri/src/projects/artifacts.rs` | Tauri path APIs |
| Plan engine | `src-tauri/src/plan_engine/` | `AppHandle` in spawn logic |

---

## Target Architecture

```
┌────────────────────────────────────────────────────────────────────────────┐
│                          PRESENTATION LAYER                                 │
│   React/Tauri  │    CLI App    │   REST API   │   Third-Party Client       │
│     (IPC)      │   (stdout)    │    (HTTP)    │      (JSON-RPC)            │
└───────────────────────────┬────────────────────────────────────────────────┘
                            │ Commands / Events (JSON)
┌───────────────────────────▼────────────────────────────────────────────────┐
│                        APPLICATION LAYER                                    │
│                    crates/loopforge-app/                                    │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                    Command/Query Dispatcher                           │  │
│  │  Commands: CreateProject, StartLoop, StopLoop, SaveDraft, RunAtomizer│  │
│  │  Queries: GetSnapshot, ListProjects, GetIterationHistory, GetStats   │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                          Services                                     │  │
│  │  ProjectService │ PlanService │ AtomizerService │ LoopSessionService │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────────────┐  │
│  │                      Ports (Traits)                                   │  │
│  │  ProjectRepository │ SessionRepository │ IterationRepository          │  │
│  │  ArtifactStore │ EventPublisher │ PathResolver │ AgentRunner          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────┬────────────────────────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────────────────────┐
│                         ADAPTER LAYER                                       │
│  ┌────────────────────────────────────────────────────────────────────┐    │
│  │ loopforge-adapters (portable)                                      │    │
│  │  SqliteRepo │ FsArtifacts │ ChannelEvents │ XdgPaths │ InMemoryRepo│    │
│  └────────────────────────────────────────────────────────────────────┘    │
│  ┌────────────────────────────────────────────────────────────────────┐    │
│  │ src-tauri/adapters (Tauri-specific)                                │    │
│  │  TauriPathResolver │ TauriEventPublisher                           │    │
│  └────────────────────────────────────────────────────────────────────┘    │
└───────────────────────────┬────────────────────────────────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────────────────────┐
│                          DOMAIN LAYER                                       │
│                      crates/ralph-core/                                     │
│  LoopEngine │ PRD │ Prompt │ Verification │ Detection │ Config │ Guardrails│
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Foundation — Create `loopforge-app` Crate

**Goal**: Single application crate with all business logic and zero Tauri dependencies.

#### 1.1 Crate Structure

```
crates/loopforge-app/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── errors.rs
    ├── models/
    │   ├── mod.rs
    │   ├── project.rs
    │   ├── session.rs
    │   ├── iteration.rs
    │   └── events.rs
    ├── ports/
    │   ├── mod.rs
    │   ├── repository.rs      # ProjectRepository, SessionRepository, IterationRepository
    │   ├── artifacts.rs       # ArtifactStore
    │   ├── events.rs          # EventPublisher
    │   ├── paths.rs           # PathResolver
    │   └── agents.rs          # AgentRegistry, AgentRunner
    └── services/
        ├── mod.rs
        ├── project.rs
        ├── plan.rs
        ├── atomizer.rs
        ├── loop_session.rs
        └── ask.rs
```

#### 1.2 Key Files

**`crates/loopforge-app/Cargo.toml`**:
```toml
[package]
name = "loopforge-app"
version = "0.1.0"
edition = "2021"

[dependencies]
ralph-core = { path = "../ralph-core" }
async-trait = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["sync"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }

[dev-dependencies]
tokio = { version = "1", features = ["full", "test-util"] }
```

**`crates/loopforge-app/src/ports/repository.rs`**:
```rust
use async_trait::async_trait;
use crate::models::{Project, Session, Iteration};
use crate::errors::AppError;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, project: &Project) -> Result<(), AppError>;
    async fn get(&self, id: &str) -> Result<Option<Project>, AppError>;
    async fn list_by_status(&self, status: &str) -> Result<Vec<Project>, AppError>;
    async fn list_all(&self) -> Result<Vec<Project>, AppError>;
    async fn update(&self, project: &Project) -> Result<(), AppError>;
    async fn update_status(&self, id: &str, status: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &Session) -> Result<(), AppError>;
    async fn get(&self, id: &str) -> Result<Option<Session>, AppError>;
    async fn get_active(&self, project_id: &str) -> Result<Option<Session>, AppError>;
    async fn end_session(&self, id: &str, ended_at: &str) -> Result<(), AppError>;
    async fn list_by_project(&self, project_id: &str, limit: usize) -> Result<Vec<Session>, AppError>;
}

#[async_trait]
pub trait IterationRepository: Send + Sync {
    async fn record(&self, iteration: &Iteration) -> Result<(), AppError>;
    async fn list_by_session(&self, session_id: &str) -> Result<Vec<Iteration>, AppError>;
    async fn list_by_project(&self, project_id: &str, limit: usize) -> Result<Vec<Iteration>, AppError>;
}
```

**`crates/loopforge-app/src/ports/events.rs`**:
```rust
use crate::models::{LoopEvent, PlanEvent, AtomizerEvent, AskEvent};

pub trait EventPublisher: Send + Sync {
    fn publish_loop(&self, project_id: &str, event: LoopEvent);
    fn publish_plan(&self, project_id: &str, event: PlanEvent);
    fn publish_atomizer(&self, project_id: &str, event: AtomizerEvent);
    fn publish_ask(&self, session_id: &str, event: AskEvent);
}
```

#### 1.3 Tasks

| # | Task | File | Priority |
|---|------|------|----------|
| 1 | Create crate manifest | `crates/loopforge-app/Cargo.toml` | P0 |
| 2 | Add to workspace | `Cargo.toml` | P0 |
| 3 | Define AppError enum | `src/errors.rs` | P0 |
| 4 | Define Project, Session, Iteration models | `src/models/*.rs` | P0 |
| 5 | Define LoopEvent, PlanEvent, etc. | `src/models/events.rs` | P0 |
| 6 | Define ProjectRepository trait | `src/ports/repository.rs` | P0 |
| 7 | Define SessionRepository trait | `src/ports/repository.rs` | P0 |
| 8 | Define IterationRepository trait | `src/ports/repository.rs` | P0 |
| 9 | Define ArtifactStore trait | `src/ports/artifacts.rs` | P0 |
| 10 | Define EventPublisher trait | `src/ports/events.rs` | P0 |
| 11 | Define PathResolver trait | `src/ports/paths.rs` | P0 |
| 12 | Define AgentRegistry trait | `src/ports/agents.rs` | P1 |
| 13 | Define AgentRunner trait | `src/ports/agents.rs` | P1 |

---

### Phase 2: Services Implementation

**Goal**: All business logic as services depending only on ports.

#### 2.1 Service Pattern

```rust
pub struct ProjectService<R: ProjectRepository, A: ArtifactStore> {
    repo: Arc<R>,
    artifacts: Arc<A>,
}

impl<R: ProjectRepository, A: ArtifactStore> ProjectService<R, A> {
    pub fn new(repo: Arc<R>, artifacts: Arc<A>) -> Self {
        Self { repo, artifacts }
    }

    pub async fn create(&self, name: &str, desc: &str, dir: &str) -> Result<Project, AppError> {
        let project = Project::new(name, desc, dir);
        self.repo.create(&project).await?;
        self.artifacts.ensure_project_dir(&project.id).await?;
        Ok(project)
    }

    pub async fn snapshot(&self, id: &str) -> Result<ProjectSnapshot, AppError> {
        let project = self.repo.get(id).await?
            .ok_or_else(|| AppError::NotFound(format!("project {}", id)))?;
        let plan = self.artifacts.read_plan(id).await?;
        let prd = self.artifacts.read_prd(id).await?;
        let config = self.artifacts.read_config(id).await?;
        let draft = self.artifacts.read_draft(id).await?;
        Ok(ProjectSnapshot { project, plan, prd, config, draft })
    }
}
```

#### 2.2 Tasks

| # | Task | File | Priority |
|---|------|------|----------|
| 14 | Implement ProjectService | `src/services/project.rs` | P0 |
| 15 | Implement LoopSessionService | `src/services/loop_session.rs` | P0 |
| 16 | Implement PlanService | `src/services/plan.rs` | P1 |
| 17 | Implement AtomizerService | `src/services/atomizer.rs` | P1 |
| 18 | Implement AskService | `src/services/ask.rs` | P2 |

---

### Phase 3: Adapter Crate

**Goal**: Reusable adapters for SQLite, filesystem, channels (non-Tauri).

#### 3.1 Crate Structure

```
crates/loopforge-adapters/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── sqlite/
    │   ├── mod.rs
    │   ├── project_repo.rs
    │   ├── session_repo.rs
    │   └── iteration_repo.rs
    ├── filesystem/
    │   ├── mod.rs
    │   └── artifact_store.rs
    ├── channel_events.rs
    ├── paths.rs
    └── inmemory/
        ├── mod.rs
        └── repo.rs
```

#### 3.2 Key Adapters

**SQLite Adapter** (`sqlite/project_repo.rs`):
```rust
use loopforge_app::ports::ProjectRepository;
use async_trait::async_trait;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct SqliteProjectRepository {
    conn: Mutex<Connection>,
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn create(&self, project: &Project) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|_| AppError::Internal("lock".into()))?;
        conn.execute(
            "INSERT INTO projects (id, name, description, status, working_directory, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![project.id, project.name, project.description,
                project.status, project.working_directory, project.created_at, project.updated_at],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
    // ...
}
```

**Channel Events** (`channel_events.rs`):
```rust
use loopforge_app::ports::EventPublisher;
use tokio::sync::broadcast;

pub struct ChannelEventPublisher {
    loop_tx: broadcast::Sender<(String, LoopEvent)>,
    plan_tx: broadcast::Sender<(String, PlanEvent)>,
    atomizer_tx: broadcast::Sender<(String, AtomizerEvent)>,
    ask_tx: broadcast::Sender<(String, AskEvent)>,
}

impl EventPublisher for ChannelEventPublisher {
    fn publish_loop(&self, project_id: &str, event: LoopEvent) {
        let _ = self.loop_tx.send((project_id.to_string(), event));
    }
    // ...
}
```

**XDG Path Resolver** (`paths.rs`):
```rust
use loopforge_app::ports::PathResolver;
use std::path::PathBuf;

pub struct XdgPathResolver {
    base_dir: PathBuf,
}

impl PathResolver for XdgPathResolver {
    fn project_dir(&self, project_id: &str) -> PathBuf {
        self.base_dir.join("projects").join(project_id)
    }
    fn app_data_dir(&self) -> PathBuf { self.base_dir.clone() }
    fn db_path(&self) -> PathBuf { self.base_dir.join("loopforge.db") }
}
```

#### 3.3 Tasks

| # | Task | File | Priority |
|---|------|------|----------|
| 19 | Create adapter crate manifest | `crates/loopforge-adapters/Cargo.toml` | P0 |
| 20 | Implement SqliteProjectRepository | `src/sqlite/project_repo.rs` | P0 |
| 21 | Implement SqliteSessionRepository | `src/sqlite/session_repo.rs` | P0 |
| 22 | Implement SqliteIterationRepository | `src/sqlite/iteration_repo.rs` | P0 |
| 23 | Implement FilesystemArtifactStore | `src/filesystem/artifact_store.rs` | P0 |
| 24 | Implement XdgPathResolver | `src/paths.rs` | P1 |
| 25 | Implement ChannelEventPublisher | `src/channel_events.rs` | P1 |
| 26 | Implement InMemoryRepository (tests) | `src/inmemory/repo.rs` | P2 |

---

### Phase 4: Tauri Adapters

**Goal**: Thin Tauri-specific adapters in `src-tauri/src/adapters/`.

#### 4.1 TauriEventPublisher

```rust
use loopforge_app::ports::EventPublisher;
use loopforge_app::models::{LoopEvent, PlanEvent, AtomizerEvent, AskEvent};
use tauri::{AppHandle, Emitter, Runtime};

pub struct TauriEventPublisher<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime + Send + Sync> EventPublisher for TauriEventPublisher<R> {
    fn publish_loop(&self, project_id: &str, event: LoopEvent) {
        let name = match &event {
            LoopEvent::SessionStarted { .. } => "loop:session-started",
            LoopEvent::IterationStarted { .. } => "loop:iteration-started",
            LoopEvent::IterationCompleted { .. } => "loop:iteration-completed",
            LoopEvent::SessionEnded { .. } => "loop:session-ended",
            LoopEvent::Output { .. } => "loop:output",
        };
        let _ = self.app.emit(name, &event);
    }
    // ...
}
```

#### 4.2 Update TauriPathResolver

Extend existing `src-tauri/src/adapters/paths_tauri.rs` to implement `loopforge_app::ports::PathResolver`.

#### 4.3 Tasks

| # | Task | File | Priority |
|---|------|------|----------|
| 27 | Implement TauriEventPublisher | `src-tauri/src/adapters/tauri_events.rs` | P0 |
| 28 | Update TauriPathResolver to implement port | `src-tauri/src/adapters/paths_tauri.rs` | P0 |

---

### Phase 5: Dependency Injection Container

**Goal**: Wire adapters to services without hardcoding.

#### 5.1 AppContext

```rust
use crate::ports::*;
use crate::services::*;
use std::sync::Arc;

pub struct AppContext<R, A, E, P>
where
    R: ProjectRepository + SessionRepository + IterationRepository + 'static,
    A: ArtifactStore + 'static,
    E: EventPublisher + 'static,
    P: PathResolver + 'static,
{
    pub project: ProjectService<R, A>,
    pub loop_session: LoopSessionService<R, R, E, DefaultAgentRunner, A>,
    pub plan: PlanService<A, E>,
    pub atomizer: AtomizerService<A, E>,
    pub ask: AskService<E>,
}

impl<R, A, E, P> AppContext<R, A, E, P>
where
    R: ProjectRepository + SessionRepository + IterationRepository + Clone + 'static,
    A: ArtifactStore + Clone + 'static,
    E: EventPublisher + Clone + 'static,
    P: PathResolver + Clone + 'static,
{
    pub fn new(repo: R, artifacts: A, events: E, paths: P) -> Self {
        let repo = Arc::new(repo);
        let artifacts = Arc::new(artifacts);
        let events = Arc::new(events);
        let paths = Arc::new(paths);

        Self {
            project: ProjectService::new(repo.clone(), artifacts.clone()),
            loop_session: LoopSessionService::new(
                repo.clone(), repo.clone(), events.clone(),
                Arc::new(DefaultAgentRunner::new(paths.clone())), artifacts.clone()
            ),
            plan: PlanService::new(artifacts.clone(), events.clone()),
            atomizer: AtomizerService::new(artifacts.clone(), events.clone()),
            ask: AskService::new(events.clone()),
        }
    }
}
```

#### 5.2 Tasks

| # | Task | File | Priority |
|---|------|------|----------|
| 29 | Create AppContext container | `crates/loopforge-app/src/container.rs` | P0 |
| 30 | Create type-erased DynAppContext | `crates/loopforge-app/src/container.rs` | P1 |

---

### Phase 6: Thin Tauri Commands

**Goal**: Commands become pure pass-through to services.

#### 6.1 Refactored Command Pattern

**Before** (coupled):
```rust
#[tauri::command]
pub async fn start_loop(app: AppHandle, args: StartLoopArgs) -> Result<String, LoopError> {
    crate::loop_manager::start_loop(app, args).await
}
```

**After** (decoupled):
```rust
#[tauri::command]
pub async fn start_loop(
    ctx: State<'_, AppContext>,
    args: StartLoopArgs,
) -> Result<String, String> {
    ctx.loop_session.start(args.into()).await.map_err(|e| e.to_string())
}
```

#### 6.2 Updated lib.rs Bootstrap

```rust
use loopforge_app::AppContext;
use loopforge_adapters::{SqliteRepository, FilesystemArtifactStore, XdgPathResolver};
use crate::adapters::{TauriPathResolver, TauriEventPublisher};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let tauri_paths = TauriPathResolver::new(app.handle())?;
            let paths = XdgPathResolver::from_base(tauri_paths.app_data_dir()?);

            let db_path = paths.db_path();
            let conn = rusqlite::Connection::open(&db_path)?;
            run_migrations(&conn)?;

            let repo = SqliteRepository::new(conn);
            let artifacts = FilesystemArtifactStore::new(Arc::new(paths.clone()));
            let events = TauriEventPublisher::new(app.handle().clone());

            let ctx = AppContext::new(repo, artifacts, events, paths);
            app.manage(ctx);

            Ok(())
        })
        .invoke_handler(generate_handlers())
        .run(tauri::generate_context!())
}
```

#### 6.3 Tasks

| # | Task | File | Priority |
|---|------|------|----------|
| 31 | Refactor `create_project` command | `src-tauri/src/commands/projects_lifecycle.rs` | P0 |
| 32 | Refactor `start_loop` command | `src-tauri/src/commands/execution.rs` | P0 |
| 33 | Refactor `stop_loop` command | `src-tauri/src/commands/execution.rs` | P0 |
| 34 | Refactor `session_stats` command | `src-tauri/src/commands/execution.rs` | P0 |
| 35 | Refactor `get_iteration_history` command | `src-tauri/src/commands/execution.rs` | P0 |
| 36 | Refactor `start_plan` command | `src-tauri/src/commands/planning.rs` | P1 |
| 37 | Refactor `run_atomizer` command | `src-tauri/src/commands/atomization.rs` | P1 |
| 38 | Refactor wizard commands | `src-tauri/src/commands/projects_wizard.rs` | P1 |
| 39 | Refactor artifact commands | `src-tauri/src/commands/projects_artifacts.rs` | P1 |
| 40 | Update lib.rs bootstrap | `src-tauri/src/lib.rs` | P0 |

---

### Phase 7: API Contract

**Goal**: Stable, versioned API contract for external consumers.

#### 7.1 Crate Structure

```
crates/loopforge-api/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── methods.rs
    └── events.rs
```

#### 7.2 Method Catalog

```rust
pub const API_VERSION: &str = "1.0.0";

pub mod methods {
    pub const PROJECT_CREATE: &str = "project.create";
    pub const PROJECT_LIST: &str = "project.list";
    pub const PROJECT_SNAPSHOT: &str = "project.snapshot";
    pub const PROJECT_ARCHIVE: &str = "project.archive";
    pub const LOOP_START: &str = "loop.start";
    pub const LOOP_STOP: &str = "loop.stop";
    pub const LOOP_STATS: &str = "loop.stats";
    pub const PLAN_START: &str = "plan.start";
    pub const PLAN_WRITE: &str = "plan.write";
    pub const PLAN_STOP: &str = "plan.stop";
    pub const ATOMIZER_RUN: &str = "atomizer.run";
    pub const ASK_QUESTION: &str = "ask.question";
    pub const ASK_HISTORY: &str = "ask.history";
}

pub mod events {
    pub const LOOP_SESSION_STARTED: &str = "loop:session-started";
    pub const LOOP_ITERATION_STARTED: &str = "loop:iteration-started";
    pub const LOOP_ITERATION_COMPLETED: &str = "loop:iteration-completed";
    pub const LOOP_SESSION_ENDED: &str = "loop:session-ended";
    pub const LOOP_OUTPUT: &str = "loop:output";
    pub const PLAN_ACTIVITY: &str = "plan:activity";
    pub const ATOMIZER_PROGRESS: &str = "atomizer:progress";
    pub const ASK_RESPONSE: &str = "ask:response";
}
```

#### 7.3 Tasks

| # | Task | File | Priority |
|---|------|------|----------|
| 41 | Create API crate manifest | `crates/loopforge-api/Cargo.toml` | P2 |
| 42 | Define method constants | `src/methods.rs` | P2 |
| 43 | Define event constants | `src/events.rs` | P2 |
| 44 | Define JSON-RPC request/response types | `src/lib.rs` | P2 |

---

## Migration Strategy

### Parallel Implementation (Non-Breaking)

1. **Phase 1-2**: Create new crates, implement ports/services alongside existing code
2. **Phase 3-4**: Implement adapters with tests
3. **Phase 5-6**: Migrate commands one-by-one using feature flags
4. **Phase 7**: Add API contract, remove legacy code

### Feature Flag Approach

```rust
#[tauri::command]
pub async fn start_loop(
    ctx: State<'_, AppContext>,
    args: StartLoopArgs,
) -> Result<String, String> {
    #[cfg(feature = "decoupled")]
    { ctx.loop_session.start(args.into()).await.map_err(|e| e.to_string()) }

    #[cfg(not(feature = "decoupled"))]
    { legacy_start_loop(args).await }
}
```

### Per-Command Migration Checklist

For each command:
1. [ ] Create equivalent service method in `loopforge-app`
2. [ ] Write unit tests with in-memory adapters
3. [ ] Update Tauri command to delegate to service
4. [ ] Update frontend IPC bindings if needed
5. [ ] Remove old implementation

---

## Final Crate Structure

```
crates/
├── loopforge-app/           # Application layer (services + ports)
│   └── src/
│       ├── errors.rs
│       ├── models/
│       ├── ports/
│       ├── services/
│       └── container.rs
├── loopforge-adapters/      # Portable infrastructure adapters
│   └── src/
│       ├── sqlite/
│       ├── filesystem/
│       ├── channel_events.rs
│       ├── paths.rs
│       └── inmemory/
├── loopforge-api/           # API contract definitions
│   └── src/
│       ├── methods.rs
│       └── events.rs
├── ralph-core/              # Domain logic (unchanged)
├── loopforge-app-core/      # → TO DELETE (merge into loopforge-app)
└── app-services/            # → TO DELETE (merge into loopforge-app)
```

---

## Affected Files

### Create

| File | Purpose |
|------|---------|
| `crates/loopforge-app/Cargo.toml` | New application crate |
| `crates/loopforge-app/src/**` | Ports, services, models (~15 files) |
| `crates/loopforge-adapters/Cargo.toml` | New adapter crate |
| `crates/loopforge-adapters/src/**` | Adapter implementations (~10 files) |
| `crates/loopforge-api/Cargo.toml` | API contract crate |
| `crates/loopforge-api/src/**` | Method/event definitions (3 files) |
| `src-tauri/src/adapters/tauri_events.rs` | Tauri event adapter |

### Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Add new workspace members |
| `src-tauri/Cargo.toml` | Add deps on new crates |
| `src-tauri/src/lib.rs` | Bootstrap with DI container |
| `src-tauri/src/commands/*.rs` | Delegate to services |
| `src-tauri/src/adapters/paths_tauri.rs` | Implement PathResolver port |

### Delete (after migration)

| File | Reason |
|------|--------|
| `crates/loopforge-app-core/` | Merged into loopforge-app |
| `crates/app-services/` | Merged into loopforge-app |
| `src-tauri/src/storage/db.rs` | Logic moved to adapters |

---

## Testing Strategy

### Unit Tests (Services)

```rust
#[tokio::test]
async fn project_service_creates_project_and_dir() {
    let repo = InMemoryRepository::new();
    let artifacts = InMemoryArtifactStore::new();
    let service = ProjectService::new(Arc::new(repo), Arc::new(artifacts));

    let project = service.create("Test", "desc", "/work").await.unwrap();

    assert!(!project.id.is_empty());
    assert!(artifacts.project_dir_exists(&project.id));
}
```

### Integration Tests (Adapters)

```rust
#[tokio::test]
async fn sqlite_repo_roundtrip() {
    let conn = Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    let repo = SqliteProjectRepository::new(conn);

    let project = Project::new("Test", "desc", "/work");
    repo.create(&project).await.unwrap();

    let loaded = repo.get(&project.id).await.unwrap();
    assert_eq!(loaded.unwrap().name, "Test");
}
```

### Contract Tests

Use existing `contract_tests/` infrastructure to verify JSON schemas.

---

## Success Criteria

- [ ] All business logic in `loopforge-app` services
- [ ] Zero Tauri imports in `loopforge-app` and `loopforge-adapters`
- [ ] All adapters implement port traits
- [ ] Existing frontend works unchanged
- [ ] CLI can execute commands without Tauri
- [ ] 90%+ test coverage on services
- [ ] API contract documented and versioned

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking frontend during migration | Parallel implementation, feature flags |
| Performance regression | Benchmark critical paths before/after |
| Incomplete migration | Track per-command progress |
| Type mismatches at boundaries | Shared DTOs in loopforge-api |
| Over-engineering | Start with minimum viable ports |

---

## References

- [Yaak Decoupling PR](https://github.com/mountain-loop/yaak/pull/354) — Real-world Tauri decoupling example
- [Tauri Specta Type-Safe IPC](https://coldfusion-example.blogspot.com/2025/12/end-to-end-type-safety-in-tauri.html) — Type-safe bindings
- [Hexagonal Architecture in Rust](http://tuttlem.github.io/2025/08/31/hexagonal-architecture-in-rust.html)
- Existing docs: `docs/BACKEND_DECOUPLING_PLAN.md`, `docs/BACKEND_DECOUPLING_IMPLEMENTATION.md`
