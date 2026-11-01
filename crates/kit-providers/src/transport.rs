//! The only door to the network.
//!
//! Every adapter talks HTTP through a [`Transport`]. [`HttpTransport`] is the
//! real one; [`ReplayTransport`] serves recorded responses so tests run
//! offline and can assert which URLs were called.

use crate::{ProviderError, retry};
use async_trait::async_trait;
use futures::stream::{self, BoxStream, StreamExt};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

/// Response body bytes, as they arrive.
pub type ByteStream = BoxStream<'static, Result<Vec<u8>, ProviderError>>;

/// HTTP for adapters.
#[async_trait]
pub trait Transport: Send + Sync {
    /// POST JSON and stream the response body. A non-2xx status is an error.
    async fn post_stream(
        &self,
        url: &str,
        headers: &[(String, String)],
        body: &Value,
    ) -> Result<ByteStream, ProviderError>;
    /// GET and parse a JSON body.
    async fn get_json(
        &self,
        url: &str,
        headers: &[(String, String)],
    ) -> Result<Value, ProviderError>;
}

/// reqwest with rustls (ring) and the OS certificate store.
pub struct HttpTransport {
    client: reqwest::Client,
}

impl HttpTransport {
    /// A client with Kit's user agent and connect timeout. Proxies come from
    /// the usual `HTTPS_PROXY` / system settings.
    pub fn new() -> Result<Self, ProviderError> {
        // Installing twice is harmless; the first provider stays.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .user_agent(concat!("kit/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| ProviderError::Fatal(format!("could not start the HTTP client: {e}")))?;
        Ok(Self { client })
    }
}

fn retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let v = headers.get(reqwest::header::RETRY_AFTER)?.to_str().ok()?;
    v.trim().parse::<u64>().ok().map(Duration::from_secs)
}

fn send_error(e: reqwest::Error) -> ProviderError {
    if e.is_timeout() || e.is_connect() {
        ProviderError::Transient(format!("could not reach the server: {e}"))
    } else {
        ProviderError::Fatal(e.to_string())
    }
}

async fn check(resp: reqwest::Response) -> Result<reqwest::Response, ProviderError> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let wait = retry_after(resp.headers());
    let body = resp.text().await.unwrap_or_default();
    Err(retry::classify(status.as_u16(), wait, &body))
}

#[async_trait]
impl Transport for HttpTransport {
    async fn post_stream(
        &self,
        url: &str,
        headers: &[(String, String)],
        body: &Value,
    ) -> Result<ByteStream, ProviderError> {
        let mut req = self.client.post(url).json(body);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let resp = check(req.send().await.map_err(send_error)?).await?;
        Ok(resp
            .bytes_stream()
            .map(|chunk| {
                chunk
                    .map(|b| b.to_vec())
                    .map_err(|e| ProviderError::Transient(format!("the stream broke: {e}")))
            })
            .boxed())
    }

    async fn get_json(
        &self,
        url: &str,
        headers: &[(String, String)],
    ) -> Result<Value, ProviderError> {
        let mut req = self.client.get(url);
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let resp = check(req.send().await.map_err(send_error)?).await?;
        resp.json()
            .await
            .map_err(|e| ProviderError::Fatal(format!("the server sent bad JSON: {e}")))
    }
}

/// One canned response for [`ReplayTransport`].
#[derive(Debug, Clone)]
pub enum Canned {
    /// Stream these chunks, in order.
    Stream(Vec<Vec<u8>>),
    /// Return this JSON (for GET).
    Json(Value),
    /// Fail with this error.
    Fail(ProviderError),
}

/// Serves canned responses in order and records every call.
#[derive(Debug, Default)]
pub struct ReplayTransport {
    queue: Mutex<VecDeque<Canned>>,
    calls: Mutex<Vec<(String, Value)>>,
}

impl ReplayTransport {
    /// Responses to serve, first to last.
    pub fn new(responses: Vec<Canned>) -> Self {
        Self {
            queue: Mutex::new(responses.into()),
            calls: Mutex::default(),
        }
    }

    /// A stream from one recorded SSE text, cut into small chunks so the
    /// parser sees splits mid-line.
    pub fn sse(text: &str) -> Canned {
        Canned::Stream(text.as_bytes().chunks(7).map(<[u8]>::to_vec).collect())
    }

    /// Every call so far: URL and body (`Null` for GET).
    pub fn calls(&self) -> Vec<(String, Value)> {
        self.calls.lock().unwrap().clone()
    }

    fn next(&self, url: &str, body: Value) -> Result<Canned, ProviderError> {
        self.calls.lock().unwrap().push((url.to_owned(), body));
        self.queue
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| ProviderError::Fatal(format!("no recorded response left for {url}")))
    }
}

#[async_trait]
impl Transport for ReplayTransport {
    async fn post_stream(
        &self,
        url: &str,
        _headers: &[(String, String)],
        body: &Value,
    ) -> Result<ByteStream, ProviderError> {
        match self.next(url, body.clone())? {
            Canned::Stream(chunks) => Ok(stream::iter(chunks.into_iter().map(Ok)).boxed()),
            Canned::Json(_) => Err(ProviderError::Fatal(
                "recorded JSON where a stream was expected".into(),
            )),
            Canned::Fail(e) => Err(e),
        }
    }

    async fn get_json(
        &self,
        url: &str,
        _headers: &[(String, String)],
    ) -> Result<Value, ProviderError> {
        match self.next(url, Value::Null)? {
            Canned::Json(v) => Ok(v),
            Canned::Stream(_) => Err(ProviderError::Fatal(
                "recorded stream where JSON was expected".into(),
            )),
            Canned::Fail(e) => Err(e),
        }
    }
}
