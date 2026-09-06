pub mod catchup;
pub mod checkpoint_writer;
pub mod config;
pub mod evidence;
pub mod input_bridge;
pub mod presentation_bridge;
pub mod replica_host;
pub mod session;
pub mod shutdown;

use std::fmt;

#[derive(Debug)]
pub enum ClientRuntimeError {
    Config(String),
    Session(String),
    Replica(String),
    Ipc(String),
}

impl fmt::Display for ClientRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(message) => write!(f, "config: {message}"),
            Self::Session(message) => write!(f, "session: {message}"),
            Self::Replica(message) => write!(f, "replica: {message}"),
            Self::Ipc(message) => write!(f, "ipc: {message}"),
        }
    }
}

impl std::error::Error for ClientRuntimeError {}
