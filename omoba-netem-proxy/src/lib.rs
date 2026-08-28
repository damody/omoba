pub mod config;
pub mod control;
pub mod delay;
pub mod evidence;
pub mod profile;
pub mod queue;
pub mod route;
pub mod runtime;

use thiserror::Error;

pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Error)]
pub enum NetemError {
    #[error("config: {0}")]
    Config(String),
    #[error("bind: {0}")]
    Bind(String),
    #[error("route: {0}")]
    Route(String),
    #[error("queue: {0}")]
    Queue(String),
    #[error("watchdog: {0}")]
    Watchdog(String),
    #[error("evidence: {0}")]
    Evidence(String),
}

pub type Result<T> = std::result::Result<T, NetemError>;
