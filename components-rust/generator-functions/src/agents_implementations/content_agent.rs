use common_lib::utils::{create_row, fetch_topics, generate_lesson_with_baml};
use common_lib::{AgentError, ContentAgent};
use golem_rust::agent_implementation;
use wstd::task::sleep;
use wstd::time::Duration;

use crate::ContentImpl;

// use crate::{
//     utils::{create_row, fetch_topics, generate_lesson_with_baml},
//     ContentImpl,
// };

#[agent_implementation]
impl ContentAgent for ContentImpl {
    fn new(name: String) -> Self {
        Self { _name: name }
    }

    async fn content_generator(&mut self, table: String) -> Result<Vec<String>, AgentError> {
        let term_topics = fetch_topics(table.as_str()).await?;
        let mut resp_vec: Vec<String> = Vec::new();
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
                "term: {}, class: {}, subject: {}, topic: {} - generated and stored\n",
                topic.term, topic.class, topic.subject, topic.topic
            );
            resp_vec.push(created_content);

            // Delay to avoid rate limiting
            // 1.5 seconds = 40 requests/minute (safe for most providers)
            sleep(Duration::from_millis(4000)).await;
        }
        Ok(resp_vec)
    }

    async fn test_sleep(&mut self) -> String {
        let stats = ["first print", "second print", "third print"];
        for statement in stats {
            println!("{}", statement);
            sleep(Duration::from_millis(3000)).await;
        }
        "Completed".to_string()
    }
}
