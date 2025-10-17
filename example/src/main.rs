use axum::{
    Json,
    extract::Request,
    middleware::{self, Next},
    response::Response,
};
use connect_axum::connect_rs_impl;

mod greet {
    pub mod v1 {
        include!("generated/greet.v1.connect.rs");
    }
}

use greet::v1::*;
use reqwest::StatusCode;
use serde_json::json;
use tokio::net::TcpListener;

struct MyGreetService;

#[connect_rs_impl(greet::v1::GreetService)]
impl MyGreetService {
    async fn greet(
        &self,
        request: GreetRequest,
    ) -> Result<GreetResponse, connect_axum::ConnectError> {
        Ok(GreetResponse {
            greeting: format!("Hello, {}!", request.name),
        })
    }

    async fn get_user(
        &self,
        request: GetUserRequest,
    ) -> Result<GetUserResponse, connect_axum::ConnectError> {
        let name = request.name;

        Ok(GetUserResponse {
            user: Some(User {
                id: name,
                email: "me@justme.com".to_string(),
                profile: None,
            }),
        })
    }
}

const TOKEN_HEADER: &str = "token";

// Basic token-based auth to demonstrate interceptors
async fn auth_interceptor(req: Request, next: Next) -> Result<Response, StatusCode> {
    if let Some(token) = req.headers().get(TOKEN_HEADER)
        && token == "opensesame"
    {
        let response = next.run(req).await;
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[tokio::main]
async fn main() {
    let service = MyGreetService;

    let app = service
        .into_router()
        .layer(middleware::from_fn(auth_interceptor))
        .route(
            "/health",
            axum::routing::get(|| async { Json(json!({ "ok": true })) }),
        );

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("Server listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
