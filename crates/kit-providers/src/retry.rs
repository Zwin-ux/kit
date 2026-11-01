//! HTTP errors as [`ProviderError`]s, and retries for the ones that pass.

use crate::ProviderError;
use std::future::Future;
use std::time::Duration;

/// Retries after the first attempt.
pub const RETRIES: u32 = 3;
const MAX_WAIT: Duration = Duration::from_secs(60);

/// Map an HTTP failure to the error class the loop acts on.
pub fn classify(status: u16, retry_after: Option<Duration>, body: &str) -> ProviderError {
    let lower = body.to_ascii_lowercase();
    let overflow = lower.contains("prompt is too long")
        || lower.contains("context_length_exceeded")
        || lower.contains("maximum context length")
        || lower.contains("context window");
    match status {
        401 | 403 => ProviderError::Auth,
        429 => ProviderError::RateLimited { retry_after },
        413 => ProviderError::ContextOverflow,
        400 if overflow => ProviderError::ContextOverflow,
        408 | 409 | 500..=599 => {
            ProviderError::Transient(format!("HTTP {status}: {}", short(body)))
        }
        400..=499 => ProviderError::BadRequest(format!("HTTP {status}: {}", short(body))),
        _ => ProviderError::Fatal(format!("HTTP {status}: {}", short(body))),
    }
}

fn short(body: &str) -> String {
    let one_line = body.split_whitespace().collect::<Vec<_>>().join(" ");
    match one_line.char_indices().nth(300) {
        Some((i, _)) => format!("{}…", &one_line[..i]),
        None => one_line,
    }
}

/// How long to wait before retry number `attempt` (0-based): the server's
/// `retry-after` when given, else 1 s, 2 s, 4 s plus up to 250 ms of jitter.
pub fn backoff(attempt: u32, retry_after: Option<Duration>) -> Duration {
    if let Some(wait) = retry_after {
        return wait.min(MAX_WAIT);
    }
    let base = Duration::from_secs(1u64 << attempt.min(5));
    let jitter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::from(d.subsec_millis()) % 250)
        .unwrap_or(0);
    (base + Duration::from_millis(jitter)).min(MAX_WAIT)
}

/// Run `attempt` until it succeeds, fails for good, or retries run out.
/// Only rate limits and temporary failures are retried.
pub async fn with_retries<T, F, Fut>(mut attempt: F) -> Result<T, ProviderError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ProviderError>>,
{
    let mut tries = 0;
    loop {
        match attempt().await {
            Ok(v) => return Ok(v),
            Err(e) if tries < RETRIES => {
                let wait = match &e {
                    ProviderError::RateLimited { retry_after } => backoff(tries, *retry_after),
                    ProviderError::Transient(_) => backoff(tries, None),
                    _ => return Err(e),
                };
                tokio::time::sleep(wait).await;
                tries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn statuses_map_to_classes() {
        assert_eq!(classify(401, None, ""), ProviderError::Auth);
        assert_eq!(
            classify(429, Some(Duration::from_secs(8)), ""),
            ProviderError::RateLimited {
                retry_after: Some(Duration::from_secs(8))
            }
        );
        assert_eq!(
            classify(400, None, "prompt is too long: 210000 tokens"),
            ProviderError::ContextOverflow
        );
        assert_eq!(
            classify(400, None, r#"{"code":"context_length_exceeded"}"#),
            ProviderError::ContextOverflow
        );
        assert!(matches!(
            classify(529, None, "overloaded"),
            ProviderError::Transient(_)
        ));
        assert!(matches!(
            classify(404, None, "no such model"),
            ProviderError::BadRequest(_)
        ));
    }

    #[test]
    fn backoff_honours_retry_after_and_caps_it() {
        assert_eq!(
            backoff(0, Some(Duration::from_secs(3))),
            Duration::from_secs(3)
        );
        assert_eq!(backoff(0, Some(Duration::from_secs(600))), MAX_WAIT);
        let b = backoff(2, None);
        assert!(
            b >= Duration::from_secs(4) && b < Duration::from_millis(4250),
            "{b:?}"
        );
    }

    #[tokio::test(start_paused = true)]
    async fn retries_transient_then_succeeds() {
        let calls = AtomicU32::new(0);
        let out = with_retries(|| async {
            if calls.fetch_add(1, Ordering::SeqCst) < 2 {
                Err(ProviderError::Transient("503".into()))
            } else {
                Ok(7)
            }
        })
        .await;
        assert_eq!(out, Ok(7));
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn does_not_retry_auth_and_stops_after_the_cap() {
        let calls = AtomicU32::new(0);
        let auth: Result<(), _> = with_retries(|| async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err(ProviderError::Auth)
        })
        .await;
        assert_eq!(auth, Err(ProviderError::Auth));
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let calls = AtomicU32::new(0);
        let limited: Result<(), _> = with_retries(|| async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err(ProviderError::RateLimited { retry_after: None })
        })
        .await;
        assert!(matches!(limited, Err(ProviderError::RateLimited { .. })));
        assert_eq!(calls.load(Ordering::SeqCst), RETRIES + 1);
    }
}
