// Request extraction for Connect protocol

use crate::{Code, ConnectError, ConnectRequest, Encoding, Method};
use axum::body::Body;
use axum::extract::Request;
use http::HeaderMap;

/// Extract a ConnectRequest from an axum Request
pub async fn extract_connect_request(req: Request) -> Result<ConnectRequest, ConnectError> {
    let (parts, body) = req.into_parts();

    // Determine HTTP method
    let method = match parts.method {
        http::Method::GET => Method::Get,
        http::Method::POST => Method::Post,
        _ => {
            return Err(ConnectError::new(
                Code::InvalidArgument,
                format!("Unsupported HTTP method: {}", parts.method),
            ));
        }
    };

    // Parse encoding from Content-Type header
    let encoding = parse_encoding(&parts.headers)?;

    // Parse Connect protocol headers
    let timeout_ms = parse_timeout(&parts.headers)?;
    let protocol_version = parse_protocol_version(&parts.headers);

    // Extract message bytes based on method
    let message = match method {
        Method::Post => extract_from_body(body).await?,
        Method::Get => extract_from_query(&parts.uri, encoding)?,
    };

    Ok(ConnectRequest {
        message,
        encoding,
        method,
        timeout_ms,
        protocol_version,
    })
}

/// Parse encoding from Content-Type header
fn parse_encoding(headers: &HeaderMap) -> Result<Encoding, ConnectError> {
    let content_type = headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if content_type.is_empty() {
        // Default to JSON for GET requests or when not specified
        return Ok(Encoding::Json);
    }

    if content_type.starts_with("application/connect+json")
        || content_type.starts_with("application/json")
    {
        Ok(Encoding::Json)
    } else if content_type.starts_with("application/connect+proto")
        || content_type.starts_with("application/proto")
    {
        Ok(Encoding::Proto)
    } else {
        Err(ConnectError::new(
            Code::InvalidArgument,
            format!("Unsupported content type: {}", content_type),
        ))
    }
}

/// Parse timeout from Connect-Timeout-Ms header
fn parse_timeout(headers: &HeaderMap) -> Result<Option<u64>, ConnectError> {
    if let Some(timeout_header) = headers.get("connect-timeout-ms") {
        let timeout_str = timeout_header
            .to_str()
            .map_err(|_| ConnectError::invalid_argument("Invalid timeout header"))?;

        let timeout = timeout_str
            .parse::<u64>()
            .map_err(|_| ConnectError::invalid_argument("Invalid timeout value"))?;

        Ok(Some(timeout))
    } else {
        Ok(None)
    }
}

/// Parse protocol version from Connect-Protocol-Version header
fn parse_protocol_version(headers: &HeaderMap) -> Option<String> {
    headers
        .get("connect-protocol-version")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Extract message bytes from POST body
async fn extract_from_body(body: Body) -> Result<Vec<u8>, ConnectError> {
    use http_body_util::BodyExt;

    let collected = body
        .collect()
        .await
        .map_err(|e| ConnectError::internal(format!("Failed to read body: {}", e)))?;

    Ok(collected.to_bytes().to_vec())
}

/// Extract message bytes from GET query parameters
fn extract_from_query(uri: &http::Uri, _encoding: Encoding) -> Result<Vec<u8>, ConnectError> {
    let query = uri
        .query()
        .ok_or_else(|| ConnectError::invalid_argument("GET request missing query parameters"))?;

    // Parse query string looking for "message" or "encoding" parameters
    let mut message_param = None;

    for pair in query.split('&') {
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");

        if key == "message" {
            message_param = Some(value);
            break;
        }
    }

    let message_encoded = message_param.ok_or_else(|| {
        ConnectError::invalid_argument("GET request missing 'message' query parameter")
    })?;

    // Decode from base64url (Connect spec uses base64url encoding for GET)
    decode_base64url(message_encoded)
}

/// Decode base64url encoded string
fn decode_base64url(input: &str) -> Result<Vec<u8>, ConnectError> {
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    // URL decode first (in case it's percent-encoded)
    let decoded_str = urlencoding::decode(input)
        .map_err(|e| ConnectError::invalid_argument(format!("URL decode failed: {}", e)))?;

    // Then base64url decode
    URL_SAFE_NO_PAD
        .decode(decoded_str.as_bytes())
        .map_err(|e| ConnectError::invalid_argument(format!("Base64 decode failed: {}", e)))
}
