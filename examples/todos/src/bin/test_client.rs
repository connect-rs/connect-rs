#[path = "../generated/todos.v1.connect.rs"]
mod todos;

use axum::http::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use todos::{GetTodoRequest, TodosServiceClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .default_headers(HeaderMap::from_iter([(
            HeaderName::from_static("token"),
            HeaderValue::from_static("opensesame"),
        )]))
        .build()?;

    let client = TodosServiceClient::with_client("http://localhost:3000", client);

    println!("=== Using generated client ===");
    let get_todo_request = GetTodoRequest {
        id: "get out of bed".to_string(),
    };

    let get_todo_response = client
        .get_todo(get_todo_request)
        .await
        .expect("response error");

    if let Some(todo) = get_todo_response.todo {
        println!(
            r#"TODO: (id: "{}", task: "{}", done: "{}")"#,
            todo.id,
            todo.task,
            if todo.done { "yes" } else { "no" }
        );
    }

    Ok(())
}
