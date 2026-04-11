pub mod atomic_write;
pub mod config;
pub mod detection;
pub mod errors;
pub mod events;
pub mod git;
pub mod guardrails;
pub mod health;
pub mod logger;
pub mod loop_engine;
pub mod ports;
pub mod prd;
pub mod prompt;
pub mod providers;
pub mod scheduler;
pub mod state;
pub mod verification;
pub mod worktree;

#[allow(dead_code)]
pub mod plugin;
#[allow(dead_code)]
pub mod signals;
