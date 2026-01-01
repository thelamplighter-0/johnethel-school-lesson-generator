use golem_rust::{agent_definition, Schema};
use serde::{Deserialize, Serialize};

pub mod utils;

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct AgentError {
    pub message: String,
    pub code: String,
}

// Add this struct to represent the file response
#[derive(Schema, Clone)]
pub struct PdfFile {
    pub content_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct TopicRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub agegroup: String,
    pub class: String,
    pub subject: String,
    pub term: String,
    pub topic: String,
    pub week: i32, // Note: week is a number, not a string
}

impl From<String> for AgentError {
    fn from(err: String) -> Self {
        AgentError {
            message: err,
            code: "ERROR".to_string(),
        }
    }
}

#[agent_definition]
pub trait ContentAgent {
    // The agent constructor, it's parameters identify the agent
    fn new(name: String) -> Self;

    async fn content_generator(&mut self, table: String) -> Result<Vec<String>, AgentError>;

    async fn test_sleep(&mut self) -> String;
}

#[agent_definition]
pub trait PdfAgent {
    fn new(name: String) -> Self;
    async fn pdf_generator(&mut self, class: String, subject: String, mode: String) -> PdfFile;
}
