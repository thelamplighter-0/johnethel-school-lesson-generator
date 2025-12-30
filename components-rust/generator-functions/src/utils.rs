use baml_client::apis::default_api::GenerateNigerianLessonError;
use baml_client::apis::*;
use baml_client::models::{ClassLevel, CompleteLessonContent, GenerateNigerianLessonRequest, Term};
use common_lib::{AgentError, TopicRecord};
use serde_json::Value;
use wstd::http::body::IntoBody;
use wstd::http::{Client, HeaderValue, Method, Request};

#[allow(dead_code)]
enum ContentType {
    Json,
    Text,
    Unsupported(String),
}

impl From<&str> for ContentType {
    fn from(content_type: &str) -> Self {
        if content_type.starts_with("application") && content_type.contains("json") {
            return Self::Json;
        } else if content_type.starts_with("text/plain") {
            return Self::Text;
        } else {
            return Self::Unsupported(content_type.to_string());
        }
    }
}

pub async fn create_row(
    input_row: CompleteLessonContent,
    source_id: String,
) -> Result<TopicRecord, AgentError> {
    let mut json_string = serde_json::to_value(&input_row).map_err(|e| AgentError {
        message: format!("Error converting rust struct to value: {:?}", e),
        code: "STRUCT_TO_VALUE_ERROR".to_string(),
    })?;

    json_string["source_id"] = serde_json::json!(source_id);

    let json_str = serde_json::to_string(&json_string).map_err(|e| AgentError {
        message: format!("Error parsing json to string: {:?}", e),
        code: "JSON_TO_STRING_PARSE_ERROR".to_string(),
    })?;

    // Remove quotes around source_id value in the JSON string
    let json_str = json_str.replace(
        &format!("\"source_id\":\"{}\"", source_id),
        &format!("\"source_id\":{}", source_id),
    );

    // SQL query
    let query = format!(
        "USE NS main DB contents; CREATE lesson_content CONTENT {:?}",
        json_str
    );

    let response = db_request(query).await?;
    // Response structure:
    // [0] = USE NS result (null)
    // [1] = USE DB result (null)
    // [2] = SELECT result (array of records)

    println!("response: {:?}", response);
    let records = if let Some(create_result) = response.get(0) {
        // Check if query was successful
        if let Some(status) = create_result.get("status") {
            if status != "OK" {
                return Err(AgentError {
                    message: format!("Query failed with status: {:?}", status),
                    code: "QUERY_FAILED".to_string(),
                });
            }
        }

        // Get the result array
        match create_result.get("result") {
            Some(Value::Array(arr)) => {
                if arr.is_empty() {
                    return Err(AgentError {
                        message: "Create returned empty result".to_string(),
                        code: "EMPTY_RESULT".to_string(),
                    });
                }
                // Deserialize the array of records
                serde_json::from_value(arr[0].clone()).map_err(|e| AgentError {
                    message: format!("Failed to deserialize records: {:?}", e),
                    code: "DESERIALIZE_ERROR".to_string(),
                })?
            }
            Some(other) => {
                return Err(AgentError {
                    message: format!("Unexpected result type: {:?}", other),
                    code: "UNEXPECTED_RESULT".to_string(),
                });
            }
            None => {
                return Err(AgentError {
                    message: "No 'result' field in response".to_string(),
                    code: "MISSING_RESULT".to_string(),
                });
            }
        }
    } else {
        return Err(AgentError {
            message: format!(
                "Expected at least 3 results, got {}",
                response.len(),
                // t.len()
            ),
            code: "INSUFFICIENT_RESULTS".to_string(),
        });
    };
    println!("✓ Created Successfully {:?}", records);
    Ok(records)
}

pub async fn fetch_rows(table: &str) -> Result<Vec<TopicRecord>, AgentError> {
    // SQL query
    let query = format!("USE NS main DB contents; SELECT * FROM {};", table);
    let response = db_request(query).await?;

    // Response structure:
    // [0] = USE NS result (null)
    // [1] = USE DB result (null)
    // [2] = SELECT result (array of records)

    let records = if let Some(select_result) = response.get(1) {
        // Check if query was successful
        if let Some(status) = select_result.get("status") {
            if status != "OK" {
                return Err(AgentError {
                    message: format!("Query failed with status: {:?}", status),
                    code: "QUERY_FAILED".to_string(),
                });
            }
        }

        // Get the result array
        match select_result.get("result") {
            Some(Value::Array(arr)) => {
                // Deserialize the array of records
                serde_json::from_value(Value::Array(arr.clone())).map_err(|e| AgentError {
                    message: format!("Failed to deserialize records: {:?}", e),
                    code: "DESERIALIZE_ERROR".to_string(),
                })?
            }
            Some(Value::Null) => {
                // Table is empty or doesn't exist
                println!("⚠️  Table '{}' is empty or doesn't exist", table);
                Vec::new()
            }
            Some(other) => {
                return Err(AgentError {
                    message: format!("Unexpected result type: {:?}", other),
                    code: "UNEXPECTED_RESULT".to_string(),
                });
            }
            None => {
                return Err(AgentError {
                    message: "No 'result' field in response".to_string(),
                    code: "MISSING_RESULT".to_string(),
                });
            }
        }
    } else {
        return Err(AgentError {
            message: format!(
                "Expected at least 3 results, got {}",
                response.len(),
                // t.len()
            ),
            code: "INSUFFICIENT_RESULTS".to_string(),
        });
    };
    println!("✓ Fetched {} records from {}", records.len(), table);
    Ok(records)
}

async fn db_request(query: String) -> Result<Vec<Value>, AgentError> {
    // SurrealDB REST API endpoint
    let url = "http://localhost:8000/sql";

    // Build HTTP request
    let request = Request::post(url)
        .header(
            "Accept",
            HeaderValue::from_str("application/json").map_err(|e| e.to_string())?,
        )
        .header(
            "Authorization",
            HeaderValue::from_str("Basic cm9vdDpzZWNyZXQ=").map_err(|e| e.to_string())?,
        )
        .body(query.into_body())
        .map_err(|e| AgentError {
            message: e.to_string(),
            code: "REQUEST_BUILD_ERROR".to_string(),
        })?;

    // Send request
    let response = Client::new().send(request).await.map_err(|e| AgentError {
        message: format!("HTTP request failed: {:?}", e),
        code: "CONNECTION_ERROR".to_string(),
    })?;

    // Check status
    let status = response.status();
    if !status.is_success() {
        return Err(AgentError {
            message: format!("Query failed with status: {}", status),
            code: "QUERY_ERROR".to_string(),
        });
    }

    // Parse response
    let mut body = response.into_body();
    let response_json: Vec<Value> = body.json().await.map_err(|e| AgentError {
        message: format!("Failed to parse JSON: {:?}", e),
        code: "PARSE_ERROR".to_string(),
    })?;
    Ok(response_json)
}

pub async fn generate_lesson_with_baml(
    row_input: TopicRecord,
) -> Result<CompleteLessonContent, AgentError> {
    let config = baml_client::apis::configuration::Configuration::default();
    let resp =
        generate_nigerian_lesson(&config, convert_from_topic_record_to_baml_format(row_input))
            .await
            .map_err(|e| AgentError {
                message: format!("Failed to generate content: {:?}", e),
                code: "GENERATION_ERROR".to_string(),
            })?;
    Ok(resp)
}

fn convert_from_topic_record_to_baml_format(input: TopicRecord) -> GenerateNigerianLessonRequest {
    let class_level = match input.class.as_str() {
        "Year 1" => Ok(ClassLevel::Primary1),
        "Year 2" => Ok(ClassLevel::Primary2),
        "Year 3" => Ok(ClassLevel::Primary3),
        "Year 4" => Ok(ClassLevel::Primary4),
        "Year 5" => Ok(ClassLevel::Primary5),
        _ => Err(AgentError {
            message: format!("Invalid class level: {}", input.class),
            code: "INVALID_CLASS_LEVEL".to_string(),
        }),
    };

    let term = match input.term.as_str() {
        "1st Term" | "Noel Term" => Ok(Term::First),
        "2nd Term" | "Calvary Term" => Ok(Term::Second),
        "3rd Term" | "Summer Term" => Ok(Term::Third),
        _ => Err(AgentError {
            message: format!("Invalid term: {}", input.term),
            code: "INVALID_TERM".to_string(),
        }),
    };
    println!("Conversion to baml type complete");
    GenerateNigerianLessonRequest {
        age_group: input.agegroup,
        class_level: class_level.unwrap(),
        subject: input.subject,
        term: term.unwrap(),
        topic: input.topic,
        week: input.week,
        __baml_options__: None,
    }
}

async fn generate_nigerian_lesson(
    configuration: &configuration::Configuration,
    generate_nigerian_lesson_request: GenerateNigerianLessonRequest,
) -> Result<CompleteLessonContent, Error<GenerateNigerianLessonError>> {
    let p_body_generate_nigerian_lesson_request = generate_nigerian_lesson_request;
    let uri_str = format!("{}/call/GenerateNigerianLesson", configuration.base_path);

    println!("one");

    // Serialize the request body to JSON
    let body_json = serde_json::to_string(&p_body_generate_nigerian_lesson_request)
        .map_err(|e| Error::from(e))?;
    println!("two");

    // Build the request
    let mut req_builder = Request::builder()
        .method(Method::POST)
        .uri(&uri_str)
        .header(
            "content-type",
            HeaderValue::from_str("application/json")
                .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?,
        );

    println!("three");

    // Add user agent if present
    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(
            "user-agent",
            HeaderValue::from_str(user_agent)
                .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?,
        );
    }

    println!("four");

    let req = req_builder
        .body(body_json.into_body())
        .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    println!("five");

    // Execute the request using wstd Client
    let response = Client::new()
        .send(req)
        .await
        .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

    println!("six");

    let status = response.status();

    println!("six-a");
    // // Get content type header value
    // let content_type_header = response
    //     .headers()
    //     .get("content-type")
    //     .and_then(|v| v.to_str().ok())
    //     .unwrap_or("application/json");
    // let content_type = ContentType::from(content_type_header);

    println!("six-b");
    if !status.is_success() {
        return Err(Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            status.to_string(),
        )));
    }
    println!("six-c");
    let mut body = response.into_body();
    println!("six-d");
    let response_json: CompleteLessonContent = body
        .json()
        .await
        .map_err(|e| Error::from(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    // let content = match content_type {
    //     ContentType::Json => {
    //         println!("seven");
    //         body.json::<CompleteLessonContent>().await.map_err(|e| {
    //             Error::from(serde_json::Error::io(std::io::Error::new(
    //                 std::io::ErrorKind::Interrupted,
    //                 "one kind error",
    //             )))
    //         })
    //     }
    //     ContentType::Text => {
    //         println!("eight");
    //         return Err(Error::from(serde_json::Error::io(std::io::Error::new(
    //             std::io::ErrorKind::InvalidData,
    //             "Received `text/plain` content type response that cannot be converted to `models::CompleteLessonContent`"
    //         ))));
    //     }
    //     ContentType::Unsupported(unknown_type) => {
    //         println!("nine");
    //         return Err(Error::from(serde_json::Error::io(std::io::Error::new(
    //             std::io::ErrorKind::InvalidData,
    //             format!("Received `{}` content type response that cannot be converted to `models::CompleteLessonContent`", unknown_type)
    //         ))));
    //     }
    // };

    // let entity: CompleteLessonContent = serde_json::from_str(&content).unwrap();
    Ok(response_json)
}
