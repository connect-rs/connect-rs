#[path = "./generated/todos.v1.connect.rs"]
mod todos_v1;

use tokio::net::TcpListener;

use connect_axum::connect_rs_impl;

use todos_v1::{GetTodoRequest, GetTodoResponse, Todo, TodosService};

struct TodosServer;

#[connect_rs_impl(todos_v1::TodosService)]
impl TodosServer {
    async fn get_todo(
        &self,
        req: GetTodoRequest,
    ) -> Result<GetTodoResponse, connect_axum::ConnectError> {
        Ok(GetTodoResponse {
            todo: Some(Todo {
                id: "get out of bed".to_string(),
                task: "Set the alarm, obey it, and be productive from the get-go".to_string(),
                done: false,
            }),
        })
    }
}

#[tokio::main]
async fn main() {
    let app = TodosServer.into_router();

    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();

    println!("Server listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
