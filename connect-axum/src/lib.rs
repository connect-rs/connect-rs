// connect-axum/src/lib.rs

pub mod extract;
pub mod message;
pub mod response;

pub use extract::extract_connect_request;
pub use response::{build_connect_response, build_error_response};

pub use connect_axum_macros::connect_impl;

#[derive(Debug)]
pub struct ConnectRequest {
    pub message: Vec<u8>,
    pub encoding: Encoding,
    pub method: Method,
    pub timeout_ms: Option<u64>,
    pub protocol_version: Option<String>,
}

#[derive(Debug)]
pub struct ConnectResponse {
    pub message: Vec<u8>,
    pub encoding: Encoding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Json,
    Proto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
}

#[derive(Debug)]
pub struct ConnectError {
    pub code: Code,
    pub message: String,
    pub details: Vec<ErrorDetail>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Code {
    Canceled,
    Unknown,
    InvalidArgument,
    DeadlineExceeded,
    NotFound,
    AlreadyExists,
    PermissionDenied,
    ResourceExhausted,
    FailedPrecondition,
    Aborted,
    OutOfRange,
    Unimplemented,
    Internal,
    Unavailable,
    DataLoss,
    Unauthenticated,
}

#[derive(Debug, Clone)]
pub struct ErrorDetail {
    pub type_url: String,
    pub value: Vec<u8>,
}

impl ConnectError {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(Code::Internal, message)
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(Code::InvalidArgument, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(Code::NotFound, message)
    }

    pub fn unimplemented(message: impl Into<String>) -> Self {
        Self::new(Code::Unimplemented, message)
    }
}

pub trait ConnectMessage: Send + Sync + 'static {
    fn encode_json(&self) -> Result<Vec<u8>, ConnectError>;
    fn encode_proto(&self) -> Result<Vec<u8>, ConnectError>;
    fn decode_json(bytes: &[u8]) -> Result<Self, ConnectError>
    where
        Self: Sized;
    fn decode_proto(bytes: &[u8]) -> Result<Self, ConnectError>
    where
        Self: Sized;
}
