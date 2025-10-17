#[path = "../generated/greet.v1.connect.rs"]
mod greet_v1;

use axum::http::{HeaderMap, HeaderName, HeaderValue};
use greet_v1::{GetUserRequest, GreetServiceClient};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .default_headers(HeaderMap::from_iter([(
            HeaderName::from_static("token"),
            HeaderValue::from_static("opensesame"),
        )]))
        .build()?;

    let client = GreetServiceClient::with_client("http://localhost:3000", client);

    println!("=== Using generated client ===");
    let get_user_request = GetUserRequest {
        name: "this will end up as the user ID".to_string(),
    };

    let get_user_response = client
        .get_user(get_user_request)
        .await
        .expect("response error");

    if let Some(user) = get_user_response.user {
        println!(r#"(id: "{}", email: "{}")"#, user.id, user.email);
    }

    Ok(())
}
