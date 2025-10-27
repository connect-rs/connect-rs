# connect-rs 🦀

```shell
nix develop # direnv allow

cargo build --release --package protoc-gen-connect-rs-axum

buf generate

cargo run --bin example # server
cargo run --bin test_client # client
```
