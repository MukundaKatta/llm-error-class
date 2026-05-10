//! # llm-error-class
//!
//! Classify provider error responses from Anthropic, OpenAI, Google, and
//! AWS Bedrock into a small set of retriable / non-retriable kinds.
//!
//! Every provider returns errors in its own shape — Anthropic wraps them
//! in `{"type":"error","error":{"type":"...","message":"..."}}`, OpenAI
//! in `{"error":{"type":"...","code":"..."}}`, Bedrock as one of a dozen
//! Java-exception-style names. This crate gives you a single
//! [`ErrorClass`] enum and one [`classify`] function that handles all
//! four providers via HTTP status + body.
//!
//! ## Example
//!
//! ```
//! use llm_error_class::{classify, ErrorClass};
//! let body = r#"{"error":{"type":"rate_limit_error","message":"slow down"}}"#;
//! assert_eq!(classify(429, body), ErrorClass::RateLimit);
//! assert!(classify(429, body).is_retriable());
//! ```

#![deny(missing_docs)]

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Provider-agnostic error class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ErrorClass {
    /// HTTP 429 / `rate_limit_error` / `ThrottlingException`. Retry with
    /// backoff.
    RateLimit,
    /// Provider is overloaded (Anthropic `overloaded_error`,
    /// `ServiceUnavailableException`). Retry with backoff.
    Overloaded,
    /// Generic 5xx. Retry with backoff.
    Server,
    /// Request timed out at the server or transport. Retry.
    Timeout,
    /// HTTP 401 / 403. **Do not retry.** Caller must fix credentials.
    Auth,
    /// Input exceeded the model's context window
    /// (`context_length_exceeded`, `ValidationException` with token
    /// count). **Do not retry** without shrinking the prompt.
    ContextWindow,
    /// Output blocked by a content policy filter. **Do not retry.**
    ContentPolicy,
    /// 400 with a parser-level error in the request body. **Do not
    /// retry** without fixing the request.
    Malformed,
    /// 404 (model not found, etc.). **Do not retry.**
    NotFound,
    /// HTTP 402 / `insufficient_quota` / `BillingException`. **Do not
    /// retry** until the account is funded.
    BillingQuota,
    /// Anything we did not recognize. Caller decides what to do.
    Unknown,
}

impl ErrorClass {
    /// True for classes that are worth retrying with backoff.
    ///
    /// Retriable: [`RateLimit`](Self::RateLimit),
    /// [`Overloaded`](Self::Overloaded), [`Server`](Self::Server),
    /// [`Timeout`](Self::Timeout). Everything else is terminal.
    pub fn is_retriable(self) -> bool {
        matches!(
            self,
            ErrorClass::RateLimit
                | ErrorClass::Overloaded
                | ErrorClass::Server
                | ErrorClass::Timeout
        )
    }
}

/// Classify an HTTP error response from any supported provider.
///
/// Pass the HTTP status code and the response body text. The body
/// keywords drive most of the classification; the status code is the
/// fallback when the body has no recognizable type field.
pub fn classify(status: u16, body: &str) -> ErrorClass {
    // Try body-typed signals first; they are stronger than the status.
    let lower = body.to_ascii_lowercase();

    // Anthropic / OpenAI-style type strings.
    if has(&lower, "rate_limit") || has(&lower, "throttling") || has(&lower, "too many requests") {
        return ErrorClass::RateLimit;
    }
    if has(&lower, "overloaded") || has(&lower, "serviceunavailable") {
        return ErrorClass::Overloaded;
    }
    if has(&lower, "context_length_exceeded")
        || has(&lower, "maximum context length")
        || has(&lower, "exceeds the maximum")
        || has(&lower, "context window")
    {
        return ErrorClass::ContextWindow;
    }
    if has(&lower, "content_policy") || has(&lower, "content_filter") || has(&lower, "safety") {
        return ErrorClass::ContentPolicy;
    }
    if has(&lower, "insufficient_quota") || has(&lower, "billing") || has(&lower, "credit") {
        return ErrorClass::BillingQuota;
    }
    if has(&lower, "timeout") || has(&lower, "timed out") {
        return ErrorClass::Timeout;
    }
    if has(&lower, "authentication")
        || has(&lower, "invalid api key")
        || has(&lower, "unauthorized")
        || has(&lower, "forbidden")
    {
        return ErrorClass::Auth;
    }
    if has(&lower, "not found") || has(&lower, "model not found") {
        return ErrorClass::NotFound;
    }
    if has(&lower, "invalid_request")
        || has(&lower, "validationexception")
        || has(&lower, "bad request")
        || has(&lower, "malformed")
    {
        return ErrorClass::Malformed;
    }

    // Fallback by HTTP status.
    match status {
        401 | 403 => ErrorClass::Auth,
        402 => ErrorClass::BillingQuota,
        404 => ErrorClass::NotFound,
        408 => ErrorClass::Timeout,
        429 => ErrorClass::RateLimit,
        503 => ErrorClass::Overloaded,
        500..=599 => ErrorClass::Server,
        400 => ErrorClass::Malformed,
        _ => ErrorClass::Unknown,
    }
}

fn has(haystack: &str, needle: &str) -> bool {
    haystack.contains(needle)
}
