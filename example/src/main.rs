use connect_axum::connect_impl;
use generated::greet::v1::{GreetManyRequest, GreetManyResponse};

mod generated;

use crate::generated::greet;

use crate::generated::greet::v1::{GreetRequest, GreetResponse, GreetService};

struct MyGreetService;

#[connect_impl(greet::v1::GreetService)]
impl MyGreetService {
    async fn greet(
        &self,
        request: GreetRequest,
    ) -> Result<GreetResponse, connect_axum::ConnectError> {
        Ok(GreetResponse {
            greeting: format!("Hello, {}!", request.name),
        })
    }

    async fn greet_many(
        &self,
        request: GreetManyRequest,
    ) -> Result<GreetManyResponse, connect_axum::ConnectError> {
        Ok(GreetManyResponse {
            greeting: format!("Hello to all of you, {}!", request.name),
        })
    }
}

#[tokio::main]
async fn main() {
    let service = MyGreetService;

    // The macro generates the into_router() method
    let app = service
        .into_router()
        .route("/health", axum::routing::get(|| async { "OK" }));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
