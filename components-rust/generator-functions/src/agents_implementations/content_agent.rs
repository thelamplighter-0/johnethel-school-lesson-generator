use common_lib::{AgentError, ContentAgent, TopicRecord};
use golem_rust::agent_implementation;

use crate::{
    utils::{create_row, fetch_rows, generate_lesson_with_baml},
    ContentImpl,
};

#[agent_implementation]
impl ContentAgent for ContentImpl {
    fn new(name: String) -> Self {
        Self { _name: name }
    }

    async fn content_generator(&mut self, table: String) -> Result<Vec<TopicRecord>, AgentError> {
        let term_topics = fetch_rows(table.as_str()).await?;
        let mut resp_vec: Vec<TopicRecord> = Vec::new();
        for topic in &term_topics {
            let generated_content =
                generate_lesson_with_baml(topic.clone())
                    .await
                    .map_err(|e| AgentError {
                        message: format!(
                            "Failed to generate content for topic '{}': {:?}",
                            topic.topic, e
                        ),
                        code: "CONTENT_GENERATION_ERROR".to_string(),
                    })?;
            let created_content = create_row(generated_content, topic.id.clone().unwrap())
                .await
                .map_err(|e| AgentError {
                    message: format!(
                        "Failed to create content for topic '{}': {:?}",
                        topic.topic, e
                    ),
                    code: "CONTENT_DB_UPDATE_ERROR".to_string(),
                })?;
            println!(
                "term: {}, class: {}, subject: {}, topic: {} - generated and stored",
                topic.term, topic.class, topic.subject, topic.topic
            );
            resp_vec.push(created_content);
        }
        Ok(resp_vec)
    }
}
