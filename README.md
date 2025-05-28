# juno

> [!WARNING]
> Juno does nothing correctly and is not ready for use even on funny little side projects.

A *very* experimental RPC server for Rust that is compatible with tRPC.
Inspired by [rspc](https://github.dev/specta-rs/rspc) and [tRPC](https://trpc.io/).

```rs
#[rpc(query)]
async fn get_user(user_id: u64) -> User {
    Ok(User {
        id: user_id,
        name: "Alice".to_string(),
        nick: Some("alice123".to_string()),
    })
}

#[rpc(mutation)]
async fn delete_user(user_id: u64) -> Result<User, RpcError> {
    if user_id == 1 {
        Err(RpcError::new(
            RpcStatus::InternalServerError,
            "Cannot delete user 1".to_string(),
        ))
    } else {
        Ok(User {
            id: user_id,
            name: "Deleted User".to_string(),
            nick: None,
        })
    }
}

#[tokio::main]
pub async fn main() {
    let app_state = AppState {
        shared_data: "Something super important".to_string(),
    };

    let rpc = RpcRouter::new()
        .for_state::<AppState>()
        .add(get_user)
        .add(delete_user)
        // "server.ts" contains a mock tRPC server that provides the types for the API.
        // Use it for "AppRouter", just like you would in a normal tRPC server.
        .write_client("server.ts")
        .unwrap()
        .to_router()
        .with_state(app_state);

    let app = Router::new().nest("/trpc", rpc);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

```

# todo

- Find a better name
- Publish on crates.io probably
- Handle state better
- Offload as much as possible from the macro, it does way too much right now.
- Figure out the best way to do auth
- Support for batch requests
- Subscriptions over SSE
  - `Tracked` trait for `Last-Event-Id` handling