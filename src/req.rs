//! HTTP request wrapper
//!
//! [`Req`] provides ergonomic access to request data including
//! headers, path parameters, query strings, and body.

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{
    Method, Request, Uri,
    body::{Body, Incoming},
    header,
};
use std::collections::HashMap;

use crate::{Error, Result};

/// HTTP request
pub struct Req {
    inner: Request<Incoming>,
    path_params: HashMap<String, String>,
    body_bytes: Option<Bytes>,
}

impl Req {
    /// Create from hyper request
    pub fn from_hyper(inner: Request<Incoming>) -> Self {
        Self {
            inner,
            path_params: HashMap::new(),
            body_bytes: None,
        }
    }

    /// Get the HTTP method
    pub fn method(&self) -> &Method {
        self.inner.method()
    }

    /// Get the URI
    pub fn uri(&self) -> &Uri {
        self.inner.uri()
    }

    /// Get the request path
    pub fn path(&self) -> &str {
        self.inner.uri().path()
    }

    /// Get the query string
    pub fn query(&self) -> Option<&str> {
        self.inner.uri().query()
    }

    /// Get a header value
    pub fn header(&self, name: &str) -> Option<&str> {
        self.inner.headers().get(name).and_then(|v| v.to_str().ok())
    }

    /// Get all headers
    pub fn headers(&self) -> &header::HeaderMap {
        self.inner.headers()
    }

    /// Set path parameters (used internally by router)
    pub(crate) fn set_path_params(&mut self, params: HashMap<String, String>) {
        self.path_params = params;
    }

    /// Get a path parameter by name
    pub fn param(&self, name: &str) -> Option<&str> {
        self.path_params.get(name).map(|s| s.as_str())
    }

    /// Get all path parameters
    pub fn params(&self) -> &HashMap<String, String> {
        &self.path_params
    }

    /// Read the entire body as bytes (consumes the body)
    pub async fn body_bytes(&mut self) -> Result<Bytes> {
        if let Some(bytes) = &self.body_bytes {
            return Ok(bytes.clone());
        }

        // This is a bit tricky - we need to extract the body from self.inner
        // For now, we'll just indicate this needs body consumption handling
        Err(Error::Custom(
            "Body already consumed or not available".to_string(),
        ))
    }

    /// Parse body as JSON
    pub async fn json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T> {
        let bytes = self.body_bytes().await?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Get the content type
    pub fn content_type(&self) -> Option<&str> {
        self.header(header::CONTENT_TYPE.as_str())
    }

    /// Check if the request expects JSON
    pub fn is_json(&self) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    /// Convert to hyper request
    pub fn into_hyper(self) -> Request<Incoming> {
        self.inner
    }

    pub(crate) async fn consume_body(mut self, max_body_size: u64) -> Result<Self> {
        let body = self.inner.body_mut();

        // Check body size hint for protection
        let max = body.size_hint().upper().unwrap_or(u64::MAX);
        if max > max_body_size {
            return Err(Error::payload_too_large(format!(
                "Request body too large (max: {} bytes)",
                max_body_size
            )));
        }

        let collected = body
            .collect()
            .await
            .map_err(|e| Error::Custom(format!("Failed to read body: {}", e)))?;

        self.body_bytes = Some(collected.to_bytes());
        Ok(self)
    }
}
