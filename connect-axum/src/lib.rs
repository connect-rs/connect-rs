pub mod encode;
pub mod message;
pub mod parse;

pub use encode::encode_http_response;
pub use parse::parse_connect_request;

pub use connect_axum_macros::connect_rs_impl;

const CONNECT_PROTOCOL_VERSION: &str = "connect-protocol-version";
const CONNECT_TIMEOUT_MS: &str = "connect-timeout-ms";

const APPLICATION_CONNECT_JSON: &str = "application/connect+json";
const APPLICATION_CONNECT_PROTO: &str = "application/connect+proto";
const APPLICATION_PROTO: &str = "application/proto";

pub struct ConnectRequest {
    pub message: Vec<u8>,
    pub encoding: Encoding,
    pub timeout_ms: Option<u64>,
    pub protocol_version: Option<String>,
}

pub struct ConnectResponse {
    pub message: Vec<u8>,
    pub encoding: Encoding,
}

#[derive(Clone)]
pub enum Encoding {
    Json,
    Proto,
}

#[derive(Debug)]
pub struct ConnectError {
    code: Code,
    message: String,
}

// https://connectrpc.com/docs/protocol/#error-codes
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

impl ConnectError {
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(Code::Internal, message)
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new(Code::InvalidArgument, message)
    }

    // TODO: other constructors
}

pub trait ConnectMessageProto: Send + Sync + 'static {
    fn encode_proto(&self) -> Result<Vec<u8>, ConnectError>;
    fn decode_proto(bytes: &[u8]) -> Result<Self, ConnectError>
    where
        Self: Sized;
}

pub trait ConnectMessageJson: Send + Sync + 'static {
    fn encode_json(&self) -> Result<Vec<u8>, ConnectError>;
    fn decode_json(bytes: &[u8]) -> Result<Self, ConnectError>
    where
        Self: Sized;
}
