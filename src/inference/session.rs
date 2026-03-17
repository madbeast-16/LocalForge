use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: ChatRole, content: String) -> Self {
        Self { role, content }
    }
}

#[derive(Debug, Clone)]
pub struct ChatSession {
    pub messages: Vec<ChatMessage>,
    pub system_prompt: Option<String>,
    pub max_context: usize,
}

impl ChatSession {
    pub fn new(max_context: usize) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
            max_context,
        }
    }

    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

#[derive(Debug, Clone)]
pub struct ContextUsage {
    pub used_tokens: usize,
    pub total_tokens: usize,
    pub percentage: f64,
    pub estimated_kv_vram_mb: usize,
}
