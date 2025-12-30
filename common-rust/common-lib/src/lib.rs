use golem_rust::{agent_definition, Schema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Schema)]
pub struct AgentError {
    pub message: String,
    pub code: String,
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

    async fn content_generator(&mut self, table: String) -> Result<Vec<TopicRecord>, AgentError>;
}

#[agent_definition]
pub trait PdfAgent {
    fn new(name: String) -> Self;
    fn pdf_generator(&mut self, class: String, subject: String);
}
