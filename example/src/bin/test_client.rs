use prost::Message;

#[path = "../generated/greet.v1.rs"]
mod greet_v1;

use greet_v1::{GreetRequest, GreetResponse};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = GreetRequest {
        name: "world".to_string(),
    };

    let mut body = Vec::new();
    request.encode(&mut body)?;

    let client = reqwest::Client::new();
    let response = client
        .post("http://localhost:3000/greet.v1.GreetService/GreetMany")
        .header("Content-Type", "application/connect+proto")
        .body(body)
        .send()
        .await?;

    println!("Status: {}", response.status());

    let response_bytes = response.bytes().await?;
    let greet_response = GreetResponse::decode(&response_bytes[..])?;

    println!("Response: {}", greet_response.greeting);

    Ok(())
}
