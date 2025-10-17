use crate::{APPLICATION_CONNECT_JSON, APPLICATION_CONNECT_PROTO, Code, ConnectError, Encoding};
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use http::{HeaderMap, HeaderValue, StatusCode, header::CONTENT_TYPE};
use serde::Serialize;

pub fn encode_http_response(
    message: Vec<u8>,
    encoding: Encoding,
) -> Result<Response, ConnectError> {
    let content_type = match encoding {
        Encoding::Json => APPLICATION_CONNECT_JSON,
        Encoding::Proto => APPLICATION_CONNECT_PROTO,
    };

    let headers = HeaderMap::from_iter([(CONTENT_TYPE, HeaderValue::from_static(content_type))]);

    Ok((StatusCode::OK, headers, message).into_response())
}

// https://connectrpc.com/docs/protocol/#error-codes
impl From<Code> for StatusCode {
    fn from(code: Code) -> Self {
        match code {
            Code::Canceled => StatusCode::from_u16(499).unwrap(), // "Client Closed Request" (non standard, comes from nginx?)
            Code::Unknown => StatusCode::INTERNAL_SERVER_ERROR,
            Code::InvalidArgument => StatusCode::BAD_REQUEST,
            Code::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
            Code::NotFound => StatusCode::NOT_FOUND,
            Code::AlreadyExists => StatusCode::CONFLICT,
            Code::PermissionDenied => StatusCode::FORBIDDEN,
            Code::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
            Code::FailedPrecondition => StatusCode::BAD_REQUEST,
            Code::Aborted => StatusCode::CONFLICT,
            Code::OutOfRange => StatusCode::BAD_REQUEST,
            Code::Unimplemented => StatusCode::NOT_IMPLEMENTED,
            Code::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Code::DataLoss => StatusCode::INTERNAL_SERVER_ERROR,
            Code::Unauthenticated => StatusCode::UNAUTHORIZED,
        }
    }
}

// https://connectrpc.com/docs/protocol/#error-codes
impl From<Code> for &'static str {
    fn from(code: Code) -> Self {
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
}

impl IntoResponse for ConnectError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody {
            code: &'static str,
            message: String,
        }

        let status_code: StatusCode = self.code.into();

        let error_body = ErrorBody {
            code: self.code.into(),
            message: self.message,
        };

        (status_code, Json(error_body)).into_response()
    }
}
