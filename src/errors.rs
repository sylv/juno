use crate::response::{IntoRpcResponse, RpcResponse};
use axum::http::StatusCode;
use serde_json::json;

#[derive(Debug, Clone, PartialEq)]
pub enum RpcStatus {
    ParseError,
    BadRequest,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotSupported,
    Timeout,
    Conflict,
    PreconditionFailed,
    PayloadTooLarge,
    UnsupportedMediaType,
    UnprocessableContent,
    TooManyRequests,
    ClientClosedRequest,
}

impl RpcStatus {
    pub fn to_http_status(&self) -> StatusCode {
        match self {
            RpcStatus::ParseError => StatusCode::BAD_REQUEST,
            RpcStatus::BadRequest => StatusCode::BAD_REQUEST,
            RpcStatus::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            RpcStatus::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            RpcStatus::BadGateway => StatusCode::BAD_GATEWAY,
            RpcStatus::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            RpcStatus::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
            RpcStatus::Unauthorized => StatusCode::UNAUTHORIZED,
            RpcStatus::Forbidden => StatusCode::FORBIDDEN,
            RpcStatus::NotFound => StatusCode::NOT_FOUND,
            RpcStatus::MethodNotSupported => StatusCode::METHOD_NOT_ALLOWED,
            RpcStatus::Timeout => StatusCode::REQUEST_TIMEOUT,
            RpcStatus::Conflict => StatusCode::CONFLICT,
            RpcStatus::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
            RpcStatus::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            RpcStatus::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            RpcStatus::UnprocessableContent => StatusCode::UNPROCESSABLE_ENTITY,
            RpcStatus::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            RpcStatus::ClientClosedRequest => StatusCode::REQUEST_TIMEOUT,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            RpcStatus::ParseError => "PARSE_ERROR".to_string(),
            RpcStatus::BadRequest => "BAD_REQUEST".to_string(),
            RpcStatus::InternalServerError => "INTERNAL_SERVER_ERROR".to_string(),
            RpcStatus::NotImplemented => "NOT_IMPLEMENTED".to_string(),
            RpcStatus::BadGateway => "BAD_GATEWAY".to_string(),
            RpcStatus::ServiceUnavailable => "SERVICE_UNAVAILABLE".to_string(),
            RpcStatus::GatewayTimeout => "GATEWAY_TIMEOUT".to_string(),
            RpcStatus::Unauthorized => "UNAUTHORIZED".to_string(),
            RpcStatus::Forbidden => "FORBIDDEN".to_string(),
            RpcStatus::NotFound => "NOT_FOUND".to_string(),
            RpcStatus::MethodNotSupported => "METHOD_NOT_SUPPORTED".to_string(),
            RpcStatus::Timeout => "TIMEOUT".to_string(),
            RpcStatus::Conflict => "CONFLICT".to_string(),
            RpcStatus::PreconditionFailed => "PRECONDITION_FAILED".to_string(),
            RpcStatus::PayloadTooLarge => "PAYLOAD_TOO_LARGE".to_string(),
            RpcStatus::UnsupportedMediaType => "UNSUPPORTED_MEDIA_TYPE".to_string(),
            RpcStatus::UnprocessableContent => "UNPROCESSABLE_CONTENT".to_string(),
            RpcStatus::TooManyRequests => "TOO_MANY_REQUESTS".to_string(),
            RpcStatus::ClientClosedRequest => "CLIENT_CLOSED_REQUEST".to_string(),
        }
    }

    pub fn to_rpc_code(&self) -> i16 {
        match self {
            RpcStatus::ParseError => -32700,
            RpcStatus::BadRequest => -32600,
            RpcStatus::InternalServerError => -32603,
            RpcStatus::NotImplemented => -32601,
            RpcStatus::BadGateway => -32602,
            RpcStatus::ServiceUnavailable => -32003,
            RpcStatus::GatewayTimeout => -32004,
            RpcStatus::Unauthorized => -32001,
            RpcStatus::Forbidden => -32003,
            RpcStatus::NotFound => -32004,
            RpcStatus::MethodNotSupported => -32005,
            RpcStatus::Timeout => -32008,
            RpcStatus::Conflict => -32009,
            RpcStatus::PreconditionFailed => -32012,
            RpcStatus::PayloadTooLarge => -32013,
            RpcStatus::UnsupportedMediaType => -32015,
            RpcStatus::UnprocessableContent => -32022,
            RpcStatus::TooManyRequests => -32029,
            RpcStatus::ClientClosedRequest => -32099,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RpcError {
    pub status: RpcStatus,
    pub message: String,
}

impl RpcError {
    pub fn new(status: RpcStatus, message: String) -> Self {
        Self { status, message }
    }
}

impl IntoRpcResponse for RpcError {
    fn into_rpc_response(self) -> RpcResponse {
        let status_code = self.status.to_http_status();

        RpcResponse::new(
            status_code,
            json!({
                "error": {
                    "message": self.message,
                    "code": self.status.to_rpc_code(),
                    "data": {
                        "code": self.status.to_string(),
                        "httpStatus": status_code.as_u16(),
                    }
                }
            }),
        )
    }
}

impl std::fmt::Display for RpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RpcError {{ status: {:?}, message: {} }}",
            self.status, self.message
        )
    }
}

impl std::error::Error for RpcError {}
