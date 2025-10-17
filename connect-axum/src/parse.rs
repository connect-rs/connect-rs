use std::collections::HashMap;

use crate::{
    APPLICATION_CONNECT_JSON, APPLICATION_CONNECT_PROTO, APPLICATION_PROTO,
    CONNECT_PROTOCOL_VERSION, CONNECT_TIMEOUT_MS, Code, ConnectError, ConnectRequest, Encoding,
};
use axum::body::Body;
use axum::extract::Request;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use http::header::CONTENT_TYPE;
use http::request::Parts;
use http::{HeaderMap, Method};
use http_body_util::BodyExt;
use url::form_urlencoded;

pub async fn parse_connect_request(req: Request) -> Result<ConnectRequest, ConnectError> {
    let (
        Parts {
            method,
            headers,
            uri,
            ..
        },
        body,
    ) = req.into_parts();

    let (encoding, message) = match method {
        Method::POST => extract_from_post_request(&headers, body).await?,
        Method::GET => extract_from_get_request(&uri)?,
        _ => {
            return Err(ConnectError::new(
                Code::InvalidArgument,
                format!("Unsupported HTTP method: {}", method),
            ));
        }
    };

    let timeout_ms = parse_timeout(&headers)?;
    let protocol_version = parse_protocol_version(&headers);

    Ok(ConnectRequest {
        message,
        encoding,
        timeout_ms,
        protocol_version,
    })
}

fn parse_timeout(headers: &HeaderMap) -> Result<Option<u64>, ConnectError> {
    if let Some(timeout_header) = headers.get(CONNECT_TIMEOUT_MS) {
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

fn parse_protocol_version(headers: &HeaderMap) -> Option<String> {
    headers
        .get(CONNECT_PROTOCOL_VERSION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

async fn extract_from_post_request(
    headers: &HeaderMap,
    body: Body,
) -> Result<(Encoding, Vec<u8>), ConnectError> {
    let encoding = if let Some(header) = headers.get(CONTENT_TYPE) {
        if let Ok(content_type) = header.to_str() {
            if content_type.starts_with(mime::APPLICATION_JSON.as_ref())
                || content_type.starts_with(APPLICATION_CONNECT_JSON)
            {
                Encoding::Json
            } else if content_type.starts_with(APPLICATION_PROTO)
                || content_type.starts_with(APPLICATION_CONNECT_PROTO)
            {
                Encoding::Proto
            } else {
                return Err(ConnectError::invalid_argument(format!(
                    "Unsupported content type: {content_type}"
                )));
            }
        } else {
            return Err(ConnectError::invalid_argument(format!(
                "Invalid content type header: {:?}",
                header,
            )));
        }
    } else {
        // MAYBE: don't default to JSON
        Encoding::Json
    };

    let message = body
        .collect()
        .await
        .map_err(|e| ConnectError::internal(format!("Failed to read request body: {e}")))?
        .to_bytes()
        .to_vec();

    Ok((encoding, message))
}

/// Extract encoding and message from query parameters for GET requests
/// Format: ?encoding=json&message=<encoded>&base64=1&connect=v1
fn extract_from_get_request(uri: &http::Uri) -> Result<(Encoding, Vec<u8>), ConnectError> {
    const BASE_64: &str = "base64";
    const ENCODING: &str = "encoding";
    const JSON: &str = "json";
    const MESSAGE: &str = "message";
    const PROTO: &str = "proto";

    let query = uri
        .query()
        .ok_or_else(|| ConnectError::invalid_argument("GET request missing query parameters"))?;

    let params: HashMap<String, String> = form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect();

    let encoding = params.get(ENCODING).ok_or_else(|| {
        ConnectError::invalid_argument("GET request missing 'encoding' parameter")
    })?;

    let encoding = match encoding.as_str() {
        JSON => Encoding::Json,
        PROTO => Encoding::Proto,
        other => {
            return Err(ConnectError::invalid_argument(format!(
                "Unsupported encoding: {}",
                other
            )));
        }
    };

    let message_encoded = params
        .get(MESSAGE)
        .ok_or_else(|| ConnectError::invalid_argument("GET request missing 'message' parameter"))?;

    let use_base64 = params.contains_key(BASE_64);

    let message = if use_base64 {
        decode_base64url(message_encoded)?
    } else {
        urlencoding::decode_binary(message_encoded.as_bytes()).into_owned()
    };

    Ok((encoding, message))
}

fn decode_base64url(input: &str) -> Result<Vec<u8>, ConnectError> {
    URL_SAFE_NO_PAD
        .decode(input.as_bytes())
        .map_err(|e| ConnectError::invalid_argument(format!("Base64 decode failed: {e}")))
}
