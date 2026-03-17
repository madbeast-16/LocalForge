use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerState {
    Stopped,
    Starting,
    Running,
    Error,
}

impl Default for ServerState {
    fn default() -> Self {
        ServerState::Stopped
    }
}
