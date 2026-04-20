# Backend Decoupling Implementation Plan

## Executive Summary

This plan transforms LoopForge's backend into a **fully decoupled, frontend-agnostic service layer** using **Hexagonal Architecture (Ports and Adapters)** with **CQRS patterns**. The backend will expose a stable contract that any frontend—React, CLI, mobile, or third-party—can consume without modification.

## Current State Analysis

### What's Already Good

1. **ralph-core** is a pure library crate with zero Tauri dependencies
2. **Ports pattern exists** in `crates/ralph-core/src/ports.rs` (GitOps, HealthChecker, GuardrailStore, StateStore, ActivityLogger)
3. **loopforge-app-core** defines service traits (`ProjectService`, `SessionService`, `MonitorService`)
4. **app-services** has DTOs and error types with `#[serde(default)]` for backward compatibility
5. **Event system** exists with typed events (`AppEvent`, `EventFanout`)

### Current Coupling Issues

| Issue | Location | Impact |
|-------|----------|--------|
| Commands embed Tauri types | `src-tauri/src/commands/*.rs` | Cannot reuse logic outside Tauri |
| DbState wraps `AppHandle` | `src-tauri/src/storage/db.rs` | DB layer coupled to Tauri lifecycle |
| Events use `app.emit()` | `src-tauri/src/loop_manager/*.rs` | Event dispatch tied to Tauri |
| State uses `tauri::State<T>` | All commands | DI coupled to Tauri's mechanism |
| File paths resolved via Tauri | `src-tauri/src/projects/artifacts.rs` | Path resolution not portable |

## Target Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        PRESENTATION LAYER                        │
│  React (Tauri IPC)  │  CLI  │  REST API  │  Third-party Client  │
└────────────────────────────┬────────────────────────────────────┘
                             │ JSON-RPC / Events
┌────────────────────────────▼────────────────────────────────────┐
│                      APPLICATION LAYER                           │
│  crates/loopforge-app/                                          │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Commands (CQRS Write)  │  Queries (CQRS Read)            │   │
│  │ CreateProject          │  GetProjectSnapshot              │   │
│  │ StartLoop              │  ListProjects                    │   │
│  │ StopLoop               │  GetIterationHistory             │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Services (Business Logic)                                 │   │
│  │ ProjectService │ PlanService │ AtomizerService │ LoopService│   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ Ports (Trait Definitions)                                 │   │
│  │ ProjectRepository │ SessionRepository │ ArtifactStore     │   │
│  │ EventPublisher │ PathResolver │ AgentRegistry             │   │
│  └──────────────────────────────────────────────────────────┘   │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                       ADAPTER LAYER                              │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │ SQLiteRepo   │ │ FileSystem   │ │ TauriEvents  │             │
│  │ (rusqlite)   │ │ Artifacts    │ │ (app.emit)   │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │ ChannelEvents│ │ InMemoryRepo │ │ StdoutEvents │             │
│  │ (for CLI)    │ │ (for tests)  │ │ (for CLI)    │             │
│  └──────────────┘ └──────────────┘ └──────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                             │
┌────────────────────────────▼────────────────────────────────────┐
│                        DOMAIN LAYER                              │
│  crates/ralph-core/                                             │
│  LoopEngine │ PRD │ Prompt │ Verification │ Detection │ Config  │
└─────────────────────────────────────────────────────────────────┘
```

## Implementation Phases

---

## Phase 1: Port Definitions (Foundation)

**Goal**: Define all ports (traits) the application layer needs, independent of any adapter.

### 1.1 Create `crates/loopforge-app/src/ports/repository.rs`

```rust
use async_trait::async_trait;
use crate::models::{Project, Session, Iteration};
use crate::errors::AppError;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create(&self, project: &Project) -> Result<(), AppError>;
    async fn get(&self, id: &str) -> Result<Option<Project>, AppError>;
    async fn list_by_status(&self, status: &str) -> Result<Vec<Project>, AppError>;
    async fn update(&self, project: &Project) -> Result<(), AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn create(&self, session: &Session) -> Result<(), AppError>;
    async fn get_active(&self, project_id: &str) -> Result<Option<Session>, AppError>;
    async fn end_session(&self, id: &str, ended_at: &str) -> Result<(), AppError>;
    async fn list_by_project(&self, project_id: &str, limit: usize) -> Result<Vec<Session>, AppError>;
}

#[async_trait]
pub trait IterationRepository: Send + Sync {
    async fn record(&self, iteration: &Iteration) -> Result<(), AppError>;
    async fn list_by_session(&self, session_id: &str) -> Result<Vec<Iteration>, AppError>;
    async fn stats(&self, project_id: &str) -> Result<IterationStats, AppError>;
}
```

### 1.2 Create `crates/loopforge-app/src/ports/artifacts.rs`

```rust
use async_trait::async_trait;
use crate::errors::AppError;
use std::path::PathBuf;

#[async_trait]
pub trait ArtifactStore: Send + Sync {
    async fn read_plan(&self, project_id: &str) -> Result<Option<String>, AppError>;
    async fn write_plan(&self, project_id: &str, content: &str) -> Result<(), AppError>;
    
    async fn read_prd(&self, project_id: &str) -> Result<Option<String>, AppError>;
    async fn write_prd(&self, project_id: &str, content: &str) -> Result<(), AppError>;
    
    async fn read_config(&self, project_id: &str) -> Result<Option<String>, AppError>;
    async fn write_config(&self, project_id: &str, content: &str) -> Result<(), AppError>;
    
    async fn read_draft(&self, project_id: &str) -> Result<Option<String>, AppError>;
    async fn write_draft(&self, project_id: &str, content: &str) -> Result<(), AppError>;
    
    async fn ensure_project_dir(&self, project_id: &str) -> Result<PathBuf, AppError>;
}

pub trait PathResolver: Send + Sync {
    fn project_dir(&self, project_id: &str) -> PathBuf;
    fn app_data_dir(&self) -> PathBuf;
    fn app_config_dir(&self) -> PathBuf;
}
```

### 1.3 Create `crates/loopforge-app/src/ports/events.rs`

```rust
use crate::events::{LoopEvent, PlanEvent, AtomizerEvent};

pub trait EventPublisher: Send + Sync {
    fn publish_loop(&self, event: LoopEvent);
    fn publish_plan(&self, event: PlanEvent);
    fn publish_atomizer(&self, event: AtomizerEvent);
}

pub trait EventSubscriber: Send + Sync {
    fn subscribe_loop(&self) -> Box<dyn Iterator<Item = LoopEvent> + Send>;
    fn subscribe_plan(&self) -> Box<dyn Iterator<Item = PlanEvent> + Send>;
}
```

### 1.4 Create `crates/loopforge-app/src/ports/agents.rs`

```rust
use async_trait::async_trait;
use crate::models::AgentInfo;
use crate::errors::AppError;

#[async_trait]
pub trait AgentRegistry: Send + Sync {
    async fn detect(&self) -> Result<Vec<AgentInfo>, AppError>;
    async fn refresh(&self) -> Result<Vec<AgentInfo>, AppError>;
    async fn get_capabilities(&self, agent: &str) -> Result<AgentCapabilities, AppError>;
}

#[async_trait]
pub trait AgentRunner: Send + Sync {
    async fn spawn(
        &self,
        agent: &str,
        args: &[String],
        working_dir: &std::path::Path,
    ) -> Result<AgentHandle, AppError>;
}
```

### Files to Create

| File | Purpose |
|------|---------|
| `crates/loopforge-app/Cargo.toml` | New crate manifest |
| `crates/loopforge-app/src/lib.rs` | Crate root |
| `crates/loopforge-app/src/ports/mod.rs` | Port module exports |
| `crates/loopforge-app/src/ports/repository.rs` | Data access traits |
| `crates/loopforge-app/src/ports/artifacts.rs` | File artifact traits |
| `crates/loopforge-app/src/ports/events.rs` | Event publishing traits |
| `crates/loopforge-app/src/ports/agents.rs` | Agent management traits |
| `crates/loopforge-app/src/errors.rs` | Unified error types |
| `crates/loopforge-app/src/models.rs` | Domain models (no Tauri deps) |

---

## Phase 2: Service Layer (Business Logic)

**Goal**: Implement all business logic as services that depend only on ports.

### 2.1 Create `crates/loopforge-app/src/services/project.rs`

```rust
use crate::ports::{ProjectRepository, ArtifactStore};
use crate::models::{Project, ProjectSnapshot};
use crate::errors::AppError;
use std::sync::Arc;

pub struct ProjectServiceImpl<R, A>
where
    R: ProjectRepository,
    A: ArtifactStore,
{
    repo: Arc<R>,
    artifacts: Arc<A>,
}

impl<R, A> ProjectServiceImpl<R, A>
where
    R: ProjectRepository,
    A: ArtifactStore,
{
    pub fn new(repo: Arc<R>, artifacts: Arc<A>) -> Self {
        Self { repo, artifacts }
    }

    pub async fn create(&self, name: &str, description: &str, working_dir: &str) -> Result<Project, AppError> {
        let project = Project::new(name, description, working_dir);
        self.repo.create(&project).await?;
        self.artifacts.ensure_project_dir(&project.id).await?;
        Ok(project)
    }

    pub async fn snapshot(&self, project_id: &str) -> Result<ProjectSnapshot, AppError> {
        let project = self.repo.get(project_id).await?
            .ok_or_else(|| AppError::NotFound(format!("project {}", project_id)))?;
        
        let plan = self.artifacts.read_plan(project_id).await?;
        let prd = self.artifacts.read_prd(project_id).await?;
        let config = self.artifacts.read_config(project_id).await?;
        
        Ok(ProjectSnapshot { project, plan, prd, config })
    }
}
```

### 2.2 Create `crates/loopforge-app/src/services/loop_session.rs`

```rust
use crate::ports::{SessionRepository, IterationRepository, EventPublisher, AgentRunner};
use crate::models::{StartLoopRequest, SessionStats};
use crate::events::LoopEvent;
use crate::errors::AppError;
use ralph_core::loop_engine::LoopEngine;
use std::sync::Arc;

pub struct LoopSessionServiceImpl<S, I, E, A>
where
    S: SessionRepository,
    I: IterationRepository,
    E: EventPublisher,
    A: AgentRunner,
{
    sessions: Arc<S>,
    iterations: Arc<I>,
    events: Arc<E>,
    runner: Arc<A>,
}

impl<S, I, E, A> LoopSessionServiceImpl<S, I, E, A>
where
    S: SessionRepository,
    I: IterationRepository,
    E: EventPublisher,
    A: AgentRunner,
{
    pub async fn start(&self, request: StartLoopRequest) -> Result<String, AppError> {
        // Validate no active session exists
        if self.sessions.get_active(&request.project_id).await?.is_some() {
            return Err(AppError::Conflict("session already running".into()));
        }
        
        // Create session record
        let session = Session::new(&request.project_id);
        self.sessions.create(&session).await?;
        
        // Emit event
        self.events.publish_loop(LoopEvent::SessionStarted {
            project_id: request.project_id.clone(),
            session_id: session.id.clone(),
        });
        
        // Start loop engine (spawns background task)
        // ... implementation using ralph_core::loop_engine
        
        Ok(session.id)
    }

    pub async fn stats(&self, project_id: &str) -> Result<SessionStats, AppError> {
        self.iterations.stats(project_id).await
    }
}
```

### Files to Create

| File | Purpose |
|------|---------|
| `crates/loopforge-app/src/services/mod.rs` | Service exports |
| `crates/loopforge-app/src/services/project.rs` | Project management |
| `crates/loopforge-app/src/services/plan.rs` | Plan generation |
| `crates/loopforge-app/src/services/atomizer.rs` | PRD atomization |
| `crates/loopforge-app/src/services/loop_session.rs` | Loop execution |
| `crates/loopforge-app/src/services/ask.rs` | Interactive Q&A |

---

## Phase 3: CQRS Command/Query Handlers

**Goal**: Create thin command and query handlers that orchestrate services.

### 3.1 Create `crates/loopforge-app/src/commands/mod.rs`

```rust
use crate::services::AppServices;
use crate::errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    CreateProject(CreateProjectCommand),
    StartLoop(StartLoopCommand),
    StopLoop(StopLoopCommand),
    SaveDraft(SaveDraftCommand),
    FinalizeDraft(FinalizeDraftCommand),
    RunAtomizer(RunAtomizerCommand),
    // ...
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandResult {
    ProjectCreated { id: String },
    LoopStarted { session_id: String },
    LoopStopped,
    DraftSaved,
    DraftFinalized,
    AtomizerComplete { stories_count: usize },
    // ...
}

pub async fn dispatch(services: &AppServices, command: Command) -> Result<CommandResult, AppError> {
    match command {
        Command::CreateProject(cmd) => {
            let project = services.project.create(&cmd.name, &cmd.description, &cmd.working_directory).await?;
            Ok(CommandResult::ProjectCreated { id: project.id })
        }
        Command::StartLoop(cmd) => {
            let session_id = services.loop_session.start(cmd.into()).await?;
            Ok(CommandResult::LoopStarted { session_id })
        }
        // ...
    }
}
```

### 3.2 Create `crates/loopforge-app/src/queries/mod.rs`

```rust
use crate::services::AppServices;
use crate::errors::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Query {
    ListProjects(ListProjectsQuery),
    GetProjectSnapshot(GetProjectSnapshotQuery),
    GetIterationHistory(GetIterationHistoryQuery),
    GetSessionStats(GetSessionStatsQuery),
    // ...
}

pub async fn execute<T: QueryResult>(services: &AppServices, query: Query) -> Result<T, AppError> {
    // Query dispatch implementation
}
```

---

## Phase 4: Adapter Implementations

**Goal**: Implement adapters for different environments (Tauri, CLI, tests).

### 4.1 SQLite Adapter (shared)

Create `crates/loopforge-adapters/src/sqlite/project_repo.rs`:

```rust
use loopforge_app::ports::ProjectRepository;
use loopforge_app::models::Project;
use loopforge_app::errors::AppError;
use async_trait::async_trait;
use rusqlite::Connection;
use std::sync::Mutex;

pub struct SqliteProjectRepository {
    conn: Mutex<Connection>,
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn create(&self, project: &Project) -> Result<(), AppError> {
        let conn = self.conn.lock().map_err(|_| AppError::Internal("lock poisoned".into()))?;
        conn.execute(
            "INSERT INTO projects (id, name, description, status, working_directory, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                project.id, project.name, project.description,
                project.status, project.working_directory,
                project.created_at, project.updated_at
            ],
        )?;
        Ok(())
    }
    // ... other methods
}
```

### 4.2 Tauri Event Adapter

Create `src-tauri/src/adapters/tauri_events.rs`:

```rust
use loopforge_app::ports::EventPublisher;
use loopforge_app::events::{LoopEvent, PlanEvent, AtomizerEvent};
use tauri::{AppHandle, Emitter};

pub struct TauriEventPublisher {
    app: AppHandle,
}

impl EventPublisher for TauriEventPublisher {
    fn publish_loop(&self, event: LoopEvent) {
        let event_name = match &event {
            LoopEvent::SessionStarted { .. } => "loop:session-started",
            LoopEvent::IterationCompleted { .. } => "loop:iteration-completed",
            // ...
        };
        let _ = self.app.emit(event_name, &event);
    }
    
    fn publish_plan(&self, event: PlanEvent) {
        let _ = self.app.emit("plan:activity", &event);
    }
    
    fn publish_atomizer(&self, event: AtomizerEvent) {
        let _ = self.app.emit("atomization-progress", &event);
    }
}
```

### 4.3 Channel Event Adapter (for CLI/tests)

Create `crates/loopforge-adapters/src/channel_events.rs`:

```rust
use loopforge_app::ports::EventPublisher;
use loopforge_app::events::{LoopEvent, PlanEvent, AtomizerEvent};
use std::sync::mpsc::Sender;

pub struct ChannelEventPublisher {
    loop_tx: Sender<LoopEvent>,
    plan_tx: Sender<PlanEvent>,
    atomizer_tx: Sender<AtomizerEvent>,
}

impl EventPublisher for ChannelEventPublisher {
    fn publish_loop(&self, event: LoopEvent) {
        let _ = self.loop_tx.send(event);
    }
    // ...
}
```

### 4.4 Path Resolver Adapters

Create `crates/loopforge-adapters/src/paths.rs`:

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
    
    fn app_data_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }
    
    fn app_config_dir(&self) -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| self.base_dir.clone())
            .join("loopforge")
    }
}
```

### Files to Create

| File | Purpose |
|------|---------|
| `crates/loopforge-adapters/Cargo.toml` | Adapter crate manifest |
| `crates/loopforge-adapters/src/lib.rs` | Adapter exports |
| `crates/loopforge-adapters/src/sqlite/mod.rs` | SQLite adapters |
| `crates/loopforge-adapters/src/sqlite/project_repo.rs` | Project repository |
| `crates/loopforge-adapters/src/sqlite/session_repo.rs` | Session repository |
| `crates/loopforge-adapters/src/filesystem/mod.rs` | Filesystem adapters |
| `crates/loopforge-adapters/src/filesystem/artifact_store.rs` | Artifact storage |
| `crates/loopforge-adapters/src/channel_events.rs` | Channel-based events |
| `crates/loopforge-adapters/src/paths.rs` | Path resolution |
| `src-tauri/src/adapters/tauri_events.rs` | Tauri event adapter |
| `src-tauri/src/adapters/tauri_paths.rs` | Tauri path adapter |

---

## Phase 5: Dependency Injection Container

**Goal**: Wire adapters to services without hardcoding dependencies.

### 5.1 Create `crates/loopforge-app/src/container.rs`

```rust
use crate::ports::*;
use crate::services::*;
use std::sync::Arc;

pub struct AppServices<R, A, E, P>
where
    R: ProjectRepository + SessionRepository + IterationRepository,
    A: ArtifactStore,
    E: EventPublisher,
    P: PathResolver,
{
    pub project: ProjectServiceImpl<R, A>,
    pub plan: PlanServiceImpl<A, E>,
    pub atomizer: AtomizerServiceImpl<A, E>,
    pub loop_session: LoopSessionServiceImpl<R, R, E, DefaultAgentRunner>,
}

impl<R, A, E, P> AppServices<R, A, E, P>
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
        
        Self {
            project: ProjectServiceImpl::new(repo.clone(), artifacts.clone()),
            plan: PlanServiceImpl::new(artifacts.clone(), events.clone()),
            atomizer: AtomizerServiceImpl::new(artifacts.clone(), events.clone()),
            loop_session: LoopSessionServiceImpl::new(
                repo.clone(), repo.clone(), events.clone(), Arc::new(DefaultAgentRunner)
            ),
        }
    }
}
```

### 5.2 Tauri Bootstrap

Update `src-tauri/src/lib.rs`:

```rust
use loopforge_app::AppServices;
use loopforge_adapters::{SqliteRepository, FilesystemArtifacts, XdgPathResolver};
use crate::adapters::TauriEventPublisher;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let paths = TauriPathResolver::new(app.handle());
            let repo = SqliteRepository::open(&paths.app_data_dir().join("loopforge.db"))?;
            let artifacts = FilesystemArtifacts::new(paths.clone());
            let events = TauriEventPublisher::new(app.handle().clone());
            
            let services = AppServices::new(repo, artifacts, events, paths);
            app.manage(services);
            
            Ok(())
        })
        .invoke_handler(generate_tauri_handlers!())
        .run(tauri::generate_context!())
}
```

---

## Phase 6: Thin Tauri Command Layer

**Goal**: Commands become pure pass-through to services.

### 6.1 Refactor Commands

Update `src-tauri/src/commands/execution.rs`:

```rust
use loopforge_app::{commands::StartLoopCommand, AppServices};
use tauri::State;

#[tauri::command]
pub async fn start_loop(
    services: State<'_, AppServices>,
    args: StartLoopCommand,
) -> Result<String, String> {
    services.loop_session
        .start(args.into())
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn stop_loop(
    services: State<'_, AppServices>,
    project_id: String,
) -> Result<(), String> {
    services.loop_session
        .stop(&project_id)
        .await
        .map_err(|err| err.to_string())
}
```

---

## Phase 7: API Contract (OpenAPI/JSON-RPC)

**Goal**: Define a stable, versioned API contract for external consumers.

### 7.1 Create `crates/loopforge-api/src/lib.rs`

```rust
use serde::{Deserialize, Serialize};

pub const API_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "jsonrpc")]
pub struct JsonRpcRequest {
    pub id: Option<String>,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub id: Option<String>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub mod methods {
    pub const CREATE_PROJECT: &str = "project.create";
    pub const LIST_PROJECTS: &str = "project.list";
    pub const GET_SNAPSHOT: &str = "project.snapshot";
    pub const START_LOOP: &str = "loop.start";
    pub const STOP_LOOP: &str = "loop.stop";
    pub const GET_STATS: &str = "loop.stats";
    // ...
}
```

### 7.2 Create Event Schema

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ApiEvent {
    SessionStarted { project_id: String, session_id: String },
    IterationStarted { project_id: String, story_id: String, iteration: u32 },
    IterationCompleted { project_id: String, story_id: String, result: String },
    SessionEnded { project_id: String, outcome: String },
    Output { project_id: String, stream: String, content: String },
    // ...
}
```

---

## Migration Strategy

### Step 1: Create New Crates (Non-Breaking)

1. Create `crates/loopforge-app` with port definitions
2. Create `crates/loopforge-adapters` with initial implementations
3. Add both to workspace `Cargo.toml`

### Step 2: Parallel Implementation

1. Implement services in `loopforge-app` alongside existing code
2. Create adapter implementations for SQLite and filesystem
3. Write integration tests using in-memory adapters

### Step 3: Gradual Migration

For each command module:
1. Create equivalent service method
2. Update Tauri command to delegate to service
3. Remove old implementation
4. Update frontend IPC bindings if contract changed

### Step 4: Remove Legacy Code

1. Delete old business logic from `src-tauri/src/`
2. Move remaining Tauri-specific code to adapters
3. Update documentation

---

## New Crate Structure

```
crates/
├── loopforge-app/           # Application layer (services + ports)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── errors.rs
│       ├── models.rs
│       ├── container.rs
│       ├── ports/
│       │   ├── mod.rs
│       │   ├── repository.rs
│       │   ├── artifacts.rs
│       │   ├── events.rs
│       │   └── agents.rs
│       ├── services/
│       │   ├── mod.rs
│       │   ├── project.rs
│       │   ├── plan.rs
│       │   ├── atomizer.rs
│       │   ├── loop_session.rs
│       │   └── ask.rs
│       ├── commands/
│       │   └── mod.rs
│       └── queries/
│           └── mod.rs
├── loopforge-adapters/      # Infrastructure adapters
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── sqlite/
│       │   ├── mod.rs
│       │   ├── project_repo.rs
│       │   ├── session_repo.rs
│       │   └── iteration_repo.rs
│       ├── filesystem/
│       │   ├── mod.rs
│       │   └── artifact_store.rs
│       ├── channel_events.rs
│       ├── paths.rs
│       └── inmemory/        # For testing
│           ├── mod.rs
│           └── repo.rs
├── loopforge-api/           # API contract definitions
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── methods.rs
│       └── events.rs
├── ralph-core/              # (existing) Domain logic
├── loopforge-app-core/      # (existing, to be merged into loopforge-app)
└── app-services/            # (existing, to be merged into loopforge-app)
```

---

## Affected Files Summary

### Files to Create

| Crate | Files |
|-------|-------|
| loopforge-app | 15 files (ports, services, commands, queries) |
| loopforge-adapters | 12 files (sqlite, filesystem, channel, inmemory) |
| loopforge-api | 3 files (contract definitions) |

### Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` (workspace) | Add new crates |
| `src-tauri/Cargo.toml` | Add dependencies on new crates |
| `src-tauri/src/lib.rs` | Bootstrap with DI container |
| `src-tauri/src/commands/*.rs` | Delegate to services |

### Files to Delete (after migration)

| File | Reason |
|------|--------|
| `crates/loopforge-app-core/` | Merged into loopforge-app |
| `crates/app-services/` | Merged into loopforge-app |
| `src-tauri/src/storage/db.rs` | Logic moved to adapters |

---

## Testing Strategy

### Unit Tests

- Test services with in-memory adapters
- No external dependencies required

### Integration Tests

- Test adapters against real SQLite (in-memory mode)
- Test filesystem adapters against temp directories

### Contract Tests

- Verify JSON-RPC request/response schemas
- Verify event payload schemas

### End-to-End Tests

- Test full Tauri app with real adapters
- Use existing `contract_tests/` infrastructure

---

## Benefits Achieved

1. **Frontend Agnostic**: Any frontend can consume the JSON-RPC API
2. **Testable**: Services tested in isolation with mocked ports
3. **Portable**: CLI or REST API can reuse the same services
4. **Maintainable**: Clear separation of concerns
5. **Evolvable**: Swap adapters without touching business logic
6. **Documented**: API contract serves as living documentation

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking frontend during migration | Parallel implementation, feature flags |
| Performance regression | Benchmark before/after each phase |
| Incomplete migration | Track progress per-command in this doc |
| Type mismatches at boundaries | Shared DTOs in loopforge-api |

---

## Success Criteria

- [ ] All business logic lives in `loopforge-app` services
- [ ] Zero Tauri imports in `loopforge-app`
- [ ] All adapters implement port traits
- [ ] Existing frontend works unchanged
- [ ] CLI can start a loop session without Tauri
- [ ] 90%+ test coverage on services
