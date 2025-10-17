// Response building for Connect protocol

use crate::{Code, ConnectError, Encoding};
use axum::response::{IntoResponse, Response};
use http::{HeaderMap, HeaderValue, StatusCode};

/// Build a successful Connect response
pub fn build_connect_response(
    message: Vec<u8>,
    encoding: Encoding,
) -> Result<Response, ConnectError> {
    let content_type = match encoding {
        Encoding::Json => "application/connect+json",
        Encoding::Proto => "application/connect+proto",
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static(content_type),
    );

    Ok((StatusCode::OK, headers, message).into_response())
}

/// Build an error response according to Connect protocol
pub fn build_error_response(error: ConnectError) -> Response {
    let status_code = error_code_to_http_status(error.code);

    // Build the error JSON response
    // Connect error format: {"code": "...", "message": "..."}
    let error_body = serde_json::json!({
        "code": code_to_string(error.code),
        "message": error.message,
    });

    let body = serde_json::to_vec(&error_body).unwrap_or_else(|_| b"{}".to_vec());

    let mut headers = HeaderMap::new();
    headers.insert(
        http::header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    (status_code, headers, body).into_response()
}

/// Convert Connect error code to HTTP status code
fn error_code_to_http_status(code: Code) -> StatusCode {
    match code {
        Code::Canceled => StatusCode::REQUEST_TIMEOUT,
        Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
        Code::InvalidArgument => StatusCode::BAD_REQUEST,
        Code::DeadlineExceeded => StatusCode::REQUEST_TIMEOUT,
        Code::NotFound => StatusCode::NOT_FOUND,
        Code::AlreadyExists => StatusCode::CONFLICT,
        Code::PermissionDenied => StatusCode::FORBIDDEN,
        Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
        Code::FailedPrecondition => StatusCode::PRECONDITION_FAILED,
        Code::Aborted => StatusCode::CONFLICT,
        Code::OutOfRange => StatusCode::BAD_REQUEST,
        Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
        Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
        Code::Unauthenticated => StatusCode::UNAUTHORIZED,
    }
}

/// Convert error code to Connect protocol string
fn code_to_string(code: Code) -> &'static str {
    match code {
        Code::Canceled => "canceled",
        Code::Unknown => "unknown",
        Code::InvalidArgument => "invalid_argument",
        Code::DeadlineExceeded => "deadline_exceeded",
        Code::NotFound => "not_found",
        Code::AlreadyExists => "already_exists",
        Code::PermissionDenied => "permission_denied",
        Code::ResourceExhausted => "resource_exhausted",
        Code::FailedPrecondition => "failed_precondition",
        Code::Aborted => "aborted",
        Code::OutOfRange => "out_of_range",
        Code::Unimplemented => "unimplemented",
        Code::Internal => "internal",
        Code::Unavailable => "unavailable",
        Code::DataLoss => "data_loss",
        Code::Unauthenticated => "unauthenticated",
    }
}

/// Implement IntoResponse for ConnectError so it can be returned from handlers
impl IntoResponse for ConnectError {
    fn into_response(self) -> Response {
        build_error_response(self)
    }
}
