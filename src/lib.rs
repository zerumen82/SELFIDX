// SELFIDEX v3.0 - Módulos

pub mod terminal;
pub mod shell;
pub mod llm;
pub mod utils;
pub mod agent;
pub mod autonomous;
pub mod config;
pub mod project;
pub mod permissions;
pub mod tasks;
pub mod git;

pub use terminal::{tui, CommandHistory, HistorySearchState};
pub use permissions::{PermissionContext, PermissionMode, RiskLevel};
pub use tasks::{TaskManager, TaskStatus};
pub use llm::{LlmProvider, ProviderConfig, LlmClient, ProviderInfo, GenerationConfig};
pub use git::GitManager;
