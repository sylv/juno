use crate::errors::{RpcError, RpcStatus};
use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::Value;

pub struct RpcResponse(StatusCode, Value);

impl RpcResponse {
    pub fn new(status: StatusCode, value: Value) -> Self {
        Self(status, value)
    }

    pub fn status(&self) -> StatusCode {
        self.0
    }

    pub fn value(&self) -> &Value {
        &self.1
    }
}

impl IntoResponse for RpcResponse {
    fn into_response(self) -> axum::response::Response {
        (self.0, Json(self.1)).into_response()
    }
}

pub trait IntoRpcResponse {
    fn into_rpc_response(self) -> RpcResponse;
}

impl<T: Serialize> IntoRpcResponse for Result<T, RpcError> {
    fn into_rpc_response(self) -> RpcResponse {
        match self {
            Err(err) => err.into_rpc_response(),
            Ok(value) => {
                let Ok(serialized) = serde_json::to_value(value) else {
                    return RpcError::new(
                        RpcStatus::InternalServerError,
                        "Failed to serialize response".to_string(),
                    )
                    .into_rpc_response();
                };

                RpcResponse::new(
                    StatusCode::OK,
                    serde_json::json!({
                        "result": {
                            "data": serialized
                        }
                    }),
                )
            }
        }
    }
}
