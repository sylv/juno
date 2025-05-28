use axum::{Router, extract::State};
use juno::errors::{RpcError, RpcStatus};
use juno::router::RpcRouter;
use juno::rpc;
use serde::Serialize;
use specta::Type;

#[derive(Clone)]
struct AppState {
    shared_data: String,
}

#[rpc(query)]
async fn get_user(State(state): State<AppState>, user_id: u64) -> Result<User, RpcError> {
    println!("Shared state in get_user_inner: {}", state.shared_data);
    if user_id == 1 {
        Err(RpcError::new(
            RpcStatus::InternalServerError,
            "Cannot get user 1".to_string(),
        ))
    } else {
        Ok(User {
            id: user_id,
            name: "Test user".to_string(),
            nick: None,
        })
    }
}

#[rpc(query)]
async fn get_server_time(State(state): State<AppState>) -> Result<String, RpcError> {
    println!("Shared state in get_server_time: {}", state.shared_data);
    Ok(chrono::Utc::now().to_rfc3339())
}

#[rpc(mutation)]
async fn add_numbers(first: i32, second: i32) -> Result<i32, RpcError> {
    Ok(first + second)
}

#[rpc(query)]
async fn get_api_version() -> String {
    "1.0.0".to_string()
}

#[rpc(query)]
async fn no_output() -> () {
    println!("This function has no output. Just like me! waow");
    ()
}

#[derive(Serialize, Type)]
pub struct Guild {
    pub id: u64,
    pub name: String,
    pub users: Vec<User>,
}

#[derive(Serialize, Type)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub nick: Option<String>,
}

#[tokio::main]
pub async fn main() {
    let app_state = AppState {
        shared_data: "Initial shared data".to_string(),
    };

    let rpc = RpcRouter::new()
        .for_state::<AppState>()
        .add(get_user)
        .add(get_server_time)
        .add(add_numbers)
        .add(get_api_version)
        .add(no_output)
        .write_client("client/src/@generated/server.ts")
        .unwrap()
        .to_router()
        .with_state(app_state);

    let app = Router::new().nest("/trpc", rpc);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
