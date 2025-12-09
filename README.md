# connect-rs 🦀

> [!WARNING]
> This project is very much a work in progress.
> It's in good enough shape to share with the world and receive input/scrutiny/pull requests, but definitely don't use it in production scenarios just yet.

An implementation of [**Connect**][connect] for [Rust] using the [Axum] framework under the hood.
Connect is a simpler alternative to [gRPC] built on plain old HTTP rather than magical ✨ bits like HTTP trailers, making Connect-compatible servers immediately compatible with standard tools like [cURL] and the [Fetch API][fetch].

## Example

With connect-rs, you can turn this [Protobuf] for the classic TODOs service...

```proto
syntax = "proto3";

package todos.v1;

message Todo {
  string id = 1;
  string task = 2;
  bool done = 3;
}

message GetTodoRequest {
  string id = 1;
}

message GetTodoResponse {
  Todo todo = 1;
}

service TodosService {
  rpc GetTodo(GetTodoRequest) returns (GetTodoResponse) {}
}
```

...into this implementation:

```rust
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
```

Pretty neat! 🚀
Minimal boilerplate, virtually no HTTP plumbing, and no [gRPC] magic.
Just plain old HTTP `POST`s (with the occasional `GET`).

## Try it out

To run the example in this repo, [install Nix][nix] and then:

```shell
# Activate the development environment
nix develop # direnv allow

# Build the code generator
cargo build --release --package protoc-gen-connect-rs-axum

# Generate the necessary Rust files from Protobuf
buf generate

# Run the example server
cargo run --bin example

# Run the example client
cargo run --bin test_client
```

## Install

I haven't yet provided any downloadable artifacts, so the best way to install the code generator (`protoc-gen-connect-rs-axum`) is to add it to your current shell environment with Nix...

```shell
nix shell "https://flakehub.com/f/connect-rs/connect-rs/0#protoc-gen-connect-rs-axum"
```

...or build it yourself in the repo using [cargo].

I'll create a real release process soon.

[axum]: https://github.com/tokio-rs/axum
[cargo]: https://doc.rust-lang.org/cargo
[connect]: https://connectrpc.com
[curl]: https://curl.se
[fetch]: https://developer.mozilla.org//docs/Web/API/Fetch_API
[grpc]: https://grpc.io
[nix]: https://docs.determinate.systems
[protobuf]: https://protobuf.dev
[rust]: https://rust-lang.org
