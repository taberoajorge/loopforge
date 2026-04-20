# Backend Decoupling Implementation Plan

## Executive Summary

This document provides a **prioritized, actionable implementation plan** for decoupling LoopForge's backend from any specific frontend. The architecture follows **Hexagonal Architecture (Ports and Adapters)** with influences from **Clean Architecture** and **CQRS patterns**.

The goal: The backend should be a self-contained service layer that any frontend (React/Tauri, CLI, REST API, third-party client) can consume without modification.

---

## Current State Assessment

### What's Already Done ✓

| Component | Location | Status |
|-----------|----------|--------|
| Pure domain logic | `crates/ralph-core` | ✓ Zero Tauri deps |
| Ports (traits) | `ralph-core/src/ports.rs` | ✓ GitOps, HealthChecker, GuardrailStore, StateStore, ActivityLogger |
| Service trait definitions | `crates/loopforge-app-core` | ✓ ProjectService, SessionService, MonitorService, PlanSessionService |
| DTO layer with serde defaults | `crates/app-services` | ✓ SessionStats, MonitorEvent, etc. |
| Tauri path adapter (started) | `src-tauri/src/adapters/` | ✓ TauriPathResolver |

### Current Coupling Issues

| Issue | Location | Severity |
|-------|----------|----------|
| Commands embed `tauri::State<T>` | `src-tauri/src/commands/*.rs` | HIGH |
| Commands use `AppHandle` directly | Plan engine, Loop manager | HIGH |
| DbState wraps Tauri lifecycle | `src-tauri/src/storage/db.rs` | HIGH |
| Events use `app.emit()` | Loop manager, Plan engine | MEDIUM |
| Path resolution via Tauri APIs | `projects/artifacts.rs` | MEDIUM |

---

## Target Architecture

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           PRESENTATION LAYER                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │ React/Tauri │  │   CLI App   │  │  REST API   │  │ Third-Party │          │
│  │   (IPC)     │  │  (stdout)   │  │  (HTTP)     │  │   Client    │          │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘          │
└─────────┼────────────────┼────────────────┼────────────────┼─────────────────┘
          │                │                │                │
          └────────────────┴────────────────┴────────────────┘
                                    │
                           JSON-RPC / Events
                                    │
┌───────────────────────────────────▼──────────────────────────────────────────┐
│                          APPLICATION LAYER                                    │
│                     crates/loopforge-app/                                     │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                      CQRS Dispatcher                                    │  │
│  │  Commands: CreateProject, StartLoop, StopLoop, SaveDraft, RunAtomizer  │  │
│  │  Queries: GetSnapshot, ListProjects, GetIterationHistory, GetStats     │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                         Services                                        │  │
│  │  ProjectService │ PlanService │ AtomizerService │ LoopSessionService   │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────────────────┐  │
│  │                    Ports (Trait Definitions)                            │  │
│  │  ProjectRepository │ SessionRepository │ ArtifactStore │ EventPublisher│  │
│  │  PathResolver │ AgentRegistry │ AgentRunner                             │  │
│  └────────────────────────────────────────────────────────────────────────┘  │
└───────────────────────────────────┬──────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼──────────────────────────────────────────┐
│                           ADAPTER LAYER                                       │
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │ loopforge-adapters (shared)                                              ││
│  │  ┌─────────────┐ ┌──────────────┐ ┌───────────────┐ ┌─────────────────┐  ││
│  │  │ SqliteRepo  │ │ FsArtifacts  │ │ ChannelEvents │ │ XdgPathResolver │  ││
│  │  └─────────────┘ └──────────────┘ └───────────────┘ └─────────────────┘  ││
│  └──────────────────────────────────────────────────────────────────────────┘│
│  ┌──────────────────────────────────────────────────────────────────────────┐│
│  │ src-tauri/adapters (Tauri-specific)                                      ││
│  │  ┌─────────────────┐ ┌───────────────────┐                               ││
│  │  │TauriPathResolver│ │TauriEventPublisher│                               ││
│  │  └─────────────────┘ └───────────────────┘                               ││
│  └──────────────────────────────────────────────────────────────────────────┘│
└───────────────────────────────────┬──────────────────────────────────────────┘
                                    │
┌───────────────────────────────────▼──────────────────────────────────────────┐
│                            DOMAIN LAYER                                       │
│                        crates/ralph-core/                                     │
│  LoopEngine │ PRD │ Prompt │ Verification │ Detection │ Config │ Guardrails  │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Create `loopforge-app` Crate (Foundation)

**Goal**: Single crate containing all application logic with zero Tauri dependencies.

**Duration**: Foundation work, ~30 files

#### 1.1 Create Crate Structure

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
    │   └── iteration.rs
    ├── ports/
    │   ├── mod.rs
    │   ├── repository.rs
    │   ├── artifacts.rs
    │   ├── events.rs
    │   └── agents.rs
    ├── services/
    │   ├── mod.rs
    │   ├── project.rs
    │   ├── plan.rs
    │   ├── atomizer.rs
    │   ├── loop_session.rs
    │   └── ask.rs
    ├── commands/
    │   └── mod.rs
    └── queries/
        └── mod.rs
```

#### 1.2 Dependencies (Cargo.toml)

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

#### 1.3 Port Definitions

**File: `crates/loopforge-app/src/ports/repository.rs`**

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
    async fn delete(&self, id: &str) -> Result<(), AppError>;
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

**File: `crates/loopforge-app/src/ports/artifacts.rs`**

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
    async fn read_output_log(&self, project_id: &str) -> Result<Option<String>, AppError>;
    async fn append_output(&self, project_id: &str, content: &str) -> Result<(), AppError>;
    async fn ensure_project_dir(&self, project_id: &str) -> Result<PathBuf, AppError>;
}

pub trait PathResolver: Send + Sync {
    fn project_dir(&self, project_id: &str) -> PathBuf;
    fn app_data_dir(&self) -> PathBuf;
    fn db_path(&self) -> PathBuf;
}
```

**File: `crates/loopforge-app/src/ports/events.rs`**

```rust
use crate::models::{LoopEvent, PlanEvent, AtomizerEvent, AskEvent};

pub trait EventPublisher: Send + Sync {
    fn publish_loop(&self, project_id: &str, event: LoopEvent);
    fn publish_plan(&self, project_id: &str, event: PlanEvent);
    fn publish_atomizer(&self, project_id: &str, event: AtomizerEvent);
    fn publish_ask(&self, session_id: &str, event: AskEvent);
}
```

#### 1.4 Tasks

| Task | File | Priority |
|------|------|----------|
| Create crate with Cargo.toml | `crates/loopforge-app/Cargo.toml` | P0 |
| Define unified error types | `src/errors.rs` | P0 |
| Define domain models | `src/models/*.rs` | P0 |
| Define repository ports | `src/ports/repository.rs` | P0 |
| Define artifact ports | `src/ports/artifacts.rs` | P0 |
| Define event ports | `src/ports/events.rs` | P0 |
| Define agent ports | `src/ports/agents.rs` | P0 |
| Add to workspace Cargo.toml | `Cargo.toml` | P0 |

---

### Phase 2: Implement Services

**Goal**: All business logic as services depending only on ports.

#### 2.1 ProjectService

```rust
use crate::ports::{ProjectRepository, ArtifactStore};
use crate::models::{Project, ProjectSnapshot, ProjectStatus};
use crate::errors::AppError;
use std::sync::Arc;

pub struct ProjectService<R: ProjectRepository, A: ArtifactStore> {
    repo: Arc<R>,
    artifacts: Arc<A>,
}

impl<R: ProjectRepository, A: ArtifactStore> ProjectService<R, A> {
    pub fn new(repo: Arc<R>, artifacts: Arc<A>) -> Self {
        Self { repo, artifacts }
    }

    pub async fn create(
        &self,
        name: &str,
        description: &str,
        working_dir: &str,
    ) -> Result<Project, AppError> {
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
        let draft = self.artifacts.read_draft(project_id).await?;
        
        Ok(ProjectSnapshot { project, plan, prd, config, draft })
    }

    pub async fn archive(&self, project_id: &str) -> Result<(), AppError> {
        self.repo.update_status(project_id, "archived").await
    }

    pub async fn list(&self, status: Option<&str>) -> Result<Vec<Project>, AppError> {
        match status {
            Some(s) => self.repo.list_by_status(s).await,
            None => self.repo.list_all().await,
        }
    }
}
```

#### 2.2 LoopSessionService

```rust
use crate::ports::{SessionRepository, IterationRepository, EventPublisher, AgentRunner, ArtifactStore};
use crate::models::{Session, StartLoopRequest, SessionStats, LoopEvent};
use crate::errors::AppError;
use ralph_core::config::RalphConfig;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

pub struct LoopSessionService<S, I, E, A, R>
where
    S: SessionRepository,
    I: IterationRepository,
    E: EventPublisher,
    A: AgentRunner,
    R: ArtifactStore,
{
    sessions: Arc<S>,
    iterations: Arc<I>,
    events: Arc<E>,
    runner: Arc<A>,
    artifacts: Arc<R>,
    active_handles: Arc<RwLock<HashMap<String, LoopHandle>>>,
}

impl<S, I, E, A, R> LoopSessionService<S, I, E, A, R>
where
    S: SessionRepository,
    I: IterationRepository,
    E: EventPublisher,
    A: AgentRunner,
    R: ArtifactStore,
{
    pub async fn start(&self, request: StartLoopRequest) -> Result<String, AppError> {
        if self.sessions.get_active(&request.project_id).await?.is_some() {
            return Err(AppError::Conflict("session already running".into()));
        }
        
        let session = Session::new(&request.project_id);
        self.sessions.create(&session).await?;
        
        self.events.publish_loop(&request.project_id, LoopEvent::SessionStarted {
            project_id: request.project_id.clone(),
            session_id: session.id.clone(),
        });
        
        // Build config and spawn loop engine...
        
        Ok(session.id)
    }

    pub async fn stop(&self, project_id: &str) -> Result<(), AppError> {
        let mut handles = self.active_handles.write().await;
        if let Some(handle) = handles.remove(project_id) {
            handle.abort();
        }
        
        if let Some(session) = self.sessions.get_active(project_id).await? {
            let now = chrono::Utc::now().to_rfc3339();
            self.sessions.end_session(&session.id, &now).await?;
        }
        
        self.events.publish_loop(project_id, LoopEvent::SessionEnded {
            project_id: project_id.to_string(),
            outcome: "stopped".to_string(),
        });
        
        Ok(())
    }

    pub async fn stats(&self, project_id: &str) -> Result<SessionStats, AppError> {
        let iterations = self.iterations.list_by_project(project_id, 1000).await?;
        // Calculate stats from iterations...
        Ok(SessionStats::calculate(&iterations))
    }
}
```

#### 2.3 Tasks

| Task | File | Priority |
|------|------|----------|
| Implement ProjectService | `src/services/project.rs` | P0 |
| Implement LoopSessionService | `src/services/loop_session.rs` | P0 |
| Implement PlanService | `src/services/plan.rs` | P1 |
| Implement AtomizerService | `src/services/atomizer.rs` | P1 |
| Implement AskService | `src/services/ask.rs` | P2 |

---

### Phase 3: Create `loopforge-adapters` Crate

**Goal**: Reusable adapter implementations for SQLite, filesystem, and channels.

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

#### 3.2 SQLite Adapter

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

impl SqliteProjectRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn: Mutex::new(conn) }
    }
}

#[async_trait]
impl ProjectRepository for SqliteProjectRepository {
    async fn create(&self, project: &Project) -> Result<(), AppError> {
        let conn = self.conn.lock()
            .map_err(|_| AppError::Internal("db lock poisoned".into()))?;
        
        conn.execute(
            "INSERT INTO projects (id, name, description, status, working_directory, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                project.id, project.name, project.description,
                project.status.as_str(), project.working_directory,
                project.created_at, project.updated_at
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Project>, AppError> {
        let conn = self.conn.lock()
            .map_err(|_| AppError::Internal("db lock poisoned".into()))?;
        
        let result = conn.query_row(
            "SELECT id, name, description, status, working_directory, created_at, updated_at 
             FROM projects WHERE id = ?1",
            [id],
            |row| Project::from_row(row),
        );
        
        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(AppError::Database(e.to_string())),
        }
    }
    
    // ... other methods
}
```

#### 3.3 Filesystem Artifact Store

```rust
use loopforge_app::ports::{ArtifactStore, PathResolver};
use loopforge_app::errors::AppError;
use async_trait::async_trait;
use std::sync::Arc;

pub struct FilesystemArtifactStore<P: PathResolver> {
    paths: Arc<P>,
}

impl<P: PathResolver> FilesystemArtifactStore<P> {
    pub fn new(paths: Arc<P>) -> Self {
        Self { paths }
    }

    fn artifact_path(&self, project_id: &str, filename: &str) -> std::path::PathBuf {
        self.paths.project_dir(project_id).join(filename)
    }
}

#[async_trait]
impl<P: PathResolver + Send + Sync> ArtifactStore for FilesystemArtifactStore<P> {
    async fn read_plan(&self, project_id: &str) -> Result<Option<String>, AppError> {
        let path = self.artifact_path(project_id, "plan.md");
        read_file_if_exists(&path).await
    }

    async fn write_plan(&self, project_id: &str, content: &str) -> Result<(), AppError> {
        let path = self.artifact_path(project_id, "plan.md");
        write_file(&path, content).await
    }

    async fn ensure_project_dir(&self, project_id: &str) -> Result<std::path::PathBuf, AppError> {
        let dir = self.paths.project_dir(project_id);
        tokio::fs::create_dir_all(&dir).await
            .map_err(|e| AppError::Filesystem(e.to_string()))?;
        Ok(dir)
    }
    
    // ... other methods
}
```

#### 3.4 Channel Event Publisher (for CLI/tests)

```rust
use loopforge_app::ports::EventPublisher;
use loopforge_app::models::{LoopEvent, PlanEvent, AtomizerEvent, AskEvent};
use tokio::sync::broadcast;

pub struct ChannelEventPublisher {
    loop_tx: broadcast::Sender<(String, LoopEvent)>,
    plan_tx: broadcast::Sender<(String, PlanEvent)>,
    atomizer_tx: broadcast::Sender<(String, AtomizerEvent)>,
    ask_tx: broadcast::Sender<(String, AskEvent)>,
}

impl ChannelEventPublisher {
    pub fn new(capacity: usize) -> Self {
        Self {
            loop_tx: broadcast::channel(capacity).0,
            plan_tx: broadcast::channel(capacity).0,
            atomizer_tx: broadcast::channel(capacity).0,
            ask_tx: broadcast::channel(capacity).0,
        }
    }

    pub fn subscribe_loop(&self) -> broadcast::Receiver<(String, LoopEvent)> {
        self.loop_tx.subscribe()
    }
}

impl EventPublisher for ChannelEventPublisher {
    fn publish_loop(&self, project_id: &str, event: LoopEvent) {
        let _ = self.loop_tx.send((project_id.to_string(), event));
    }

    fn publish_plan(&self, project_id: &str, event: PlanEvent) {
        let _ = self.plan_tx.send((project_id.to_string(), event));
    }

    fn publish_atomizer(&self, project_id: &str, event: AtomizerEvent) {
        let _ = self.atomizer_tx.send((project_id.to_string(), event));
    }

    fn publish_ask(&self, session_id: &str, event: AskEvent) {
        let _ = self.ask_tx.send((session_id.to_string(), event));
    }
}
```

#### 3.5 Tasks

| Task | File | Priority |
|------|------|----------|
| Create crate with Cargo.toml | `crates/loopforge-adapters/Cargo.toml` | P0 |
| Implement SqliteProjectRepository | `src/sqlite/project_repo.rs` | P0 |
| Implement SqliteSessionRepository | `src/sqlite/session_repo.rs` | P0 |
| Implement SqliteIterationRepository | `src/sqlite/iteration_repo.rs` | P0 |
| Implement FilesystemArtifactStore | `src/filesystem/artifact_store.rs` | P0 |
| Implement ChannelEventPublisher | `src/channel_events.rs` | P1 |
| Implement XdgPathResolver | `src/paths.rs` | P1 |
| Implement InMemoryRepository (tests) | `src/inmemory/repo.rs` | P2 |

---

### Phase 4: Dependency Injection Container

**Goal**: Wire adapters to services without hardcoding.

#### 4.1 AppContext

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

#### 4.2 Type-Erased Container (for Tauri State)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

pub type DynProjectRepository = Arc<dyn ProjectRepository>;
pub type DynArtifactStore = Arc<dyn ArtifactStore>;
pub type DynEventPublisher = Arc<dyn EventPublisher>;
pub type DynPathResolver = Arc<dyn PathResolver>;

pub struct DynAppContext {
    pub repo: DynProjectRepository,
    pub artifacts: DynArtifactStore,
    pub events: DynEventPublisher,
    pub paths: DynPathResolver,
    pub active_loops: Arc<RwLock<HashMap<String, LoopHandle>>>,
    pub active_plans: Arc<RwLock<HashMap<String, PlanHandle>>>,
}
```

---

### Phase 5: Tauri-Specific Adapters

**Goal**: Thin adapters for Tauri runtime.

#### 5.1 TauriEventPublisher

```rust
use loopforge_app::ports::EventPublisher;
use loopforge_app::models::{LoopEvent, PlanEvent, AtomizerEvent, AskEvent};
use tauri::{AppHandle, Emitter, Runtime};

pub struct TauriEventPublisher<R: Runtime> {
    app: AppHandle<R>,
}

impl<R: Runtime> TauriEventPublisher<R> {
    pub fn new(app: AppHandle<R>) -> Self {
        Self { app }
    }
}

impl<R: Runtime + Send + Sync> EventPublisher for TauriEventPublisher<R> {
    fn publish_loop(&self, project_id: &str, event: LoopEvent) {
        let event_name = match &event {
            LoopEvent::SessionStarted { .. } => "loop:session-started",
            LoopEvent::IterationStarted { .. } => "loop:iteration-started",
            LoopEvent::IterationCompleted { .. } => "loop:iteration-completed",
            LoopEvent::SessionEnded { .. } => "loop:session-ended",
            LoopEvent::Output { .. } => "loop:output",
        };
        let _ = self.app.emit(event_name, &event);
    }

    fn publish_plan(&self, project_id: &str, event: PlanEvent) {
        let _ = self.app.emit("plan-activity", &event);
    }

    fn publish_atomizer(&self, project_id: &str, event: AtomizerEvent) {
        let _ = self.app.emit("atomization-progress", &event);
    }

    fn publish_ask(&self, session_id: &str, event: AskEvent) {
        let _ = self.app.emit("ask-response", &event);
    }
}
```

#### 5.2 Updated TauriPathResolver

```rust
use loopforge_app::ports::PathResolver;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

pub struct TauriPathResolver<R: Runtime> {
    data_dir: PathBuf,
}

impl<R: Runtime> TauriPathResolver<R> {
    pub fn new(app: &AppHandle<R>) -> Result<Self, String> {
        let data_dir = app.path().app_data_dir()
            .map_err(|e| e.to_string())?;
        Ok(Self { data_dir })
    }
}

impl<R: Runtime + Send + Sync> PathResolver for TauriPathResolver<R> {
    fn project_dir(&self, project_id: &str) -> PathBuf {
        self.data_dir.join("projects").join(project_id)
    }

    fn app_data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    fn db_path(&self) -> PathBuf {
        self.data_dir.join("loopforge.db")
    }
}
```

---

### Phase 6: Thin Tauri Commands

**Goal**: Commands become pure delegation to services.

#### 6.1 Refactored Commands

```rust
use loopforge_app::services::ProjectService;
use loopforge_app::models::{CreateProjectRequest, ProjectSnapshot};
use tauri::State;

#[tauri::command]
pub async fn create_project(
    ctx: State<'_, AppContext>,
    name: String,
    description: String,
    working_dir: String,
) -> Result<String, String> {
    ctx.project
        .create(&name, &description, &working_dir)
        .await
        .map(|p| p.id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_project_snapshot(
    ctx: State<'_, AppContext>,
    project_id: String,
) -> Result<ProjectSnapshot, String> {
    ctx.project
        .snapshot(&project_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_loop(
    ctx: State<'_, AppContext>,
    args: StartLoopArgs,
) -> Result<String, String> {
    ctx.loop_session
        .start(args.into())
        .await
        .map_err(|e| e.to_string())
}
```

#### 6.2 Updated lib.rs Bootstrap

```rust
use loopforge_app::AppContext;
use loopforge_adapters::{
    SqliteRepository, FilesystemArtifactStore, XdgPathResolver,
};
use crate::adapters::{TauriPathResolver, TauriEventPublisher};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let tauri_paths = TauriPathResolver::new(app.handle())?;
            let paths = XdgPathResolver::from_base(tauri_paths.app_data_dir());
            
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

---

### Phase 7: API Contract

**Goal**: Stable, versioned API contract for external consumers.

#### 7.1 Create `loopforge-api` Crate

```
crates/loopforge-api/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── methods.rs
    └── events.rs
```

#### 7.2 JSON-RPC Contract

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

---

## Migration Strategy

### Parallel Implementation (Non-Breaking)

1. **Week 1-2**: Create new crates, implement ports and models
2. **Week 3-4**: Implement services with tests using in-memory adapters
3. **Week 5-6**: Implement SQLite and filesystem adapters
4. **Week 7-8**: Migrate Tauri commands one module at a time
5. **Week 9**: Remove legacy code, update documentation

### Per-Command Migration Pattern

For each command:
1. Create equivalent service method in `loopforge-app`
2. Write tests for service using in-memory adapters
3. Update Tauri command to delegate to service
4. Remove old implementation from `src-tauri`
5. Update frontend IPC bindings if contract changed

### Feature Flag Approach

```rust
#[tauri::command]
pub async fn start_loop(
    ctx: State<'_, AppContext>,
    args: StartLoopArgs,
) -> Result<String, String> {
    #[cfg(feature = "new-services")]
    {
        ctx.loop_session.start(args.into()).await.map_err(|e| e.to_string())
    }
    
    #[cfg(not(feature = "new-services"))]
    {
        legacy_start_loop(args).await
    }
}
```

---

## New Crate Structure (Final)

```
crates/
├── loopforge-app/           # Application layer (services + ports)
│   └── src/
│       ├── errors.rs
│       ├── models/
│       ├── ports/
│       ├── services/
│       ├── commands/
│       └── queries/
├── loopforge-adapters/      # Shared infrastructure adapters
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
├── loopforge-app-core/      # → Merge into loopforge-app
└── app-services/            # → Merge into loopforge-app
```

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

Verify JSON-RPC schemas match frontend expectations using the existing `contract_tests/` infrastructure.

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
| Incomplete migration | Track per-command progress in this doc |
| Type mismatches at boundaries | Shared DTOs in loopforge-api |
| Over-engineering | Start with minimum viable ports |

---

## References

- [Hexagonal Architecture in Rust](http://tuttlem.github.io/2025/08/31/hexagonal-architecture-in-rust.html)
- [Clean Architecture for Tauri + Rust + Svelte](https://lobehub.com/en/skills/carlomicieli-rusty-shed-clean-architecture)
- [Tauri v2 IPC Layer Architecture](https://coldfusion-example.blogspot.com/2026/01/tauri-v2-vs-electron-rewriting-ipc.html)
- [TauRPC - Typesafe IPC for Tauri](https://lib.rs/crates/taurpc)
- Existing plan: `docs/BACKEND_DECOUPLING_PLAN.md`
