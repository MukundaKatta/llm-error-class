# llm-error-class

[![crates.io](https://img.shields.io/crates/v/llm-error-class.svg)](https://crates.io/crates/llm-error-class)
[![docs.rs](https://img.shields.io/docsrs/llm-error-class)](https://docs.rs/llm-error-class)

Classify LLM provider error responses into a small enum of retriable vs.
terminal kinds. Covers Anthropic, OpenAI, Google Gemini, and AWS Bedrock.
Zero deps.

## Why

Every retry-wrapper you write needs the same logic: is this a 5xx I
should back off on, a 429 I should slow down for, or a 400 that means
"don't try this again"? Provider error shapes are inconsistent.
`classify(status, body)` gives you one answer.

## Usage

```rust
use llm_error_class::{classify, ErrorClass};

let body = r#"{"error":{"type":"rate_limit_error","message":"slow down"}}"#;
let kind = classify(429, body);
assert_eq!(kind, ErrorClass::RateLimit);

if kind.is_retriable() {
    // back off and retry
}
```

## Classes

| Class | Retriable | Examples |
| --- | --- | --- |
| `RateLimit` | yes | HTTP 429, `rate_limit_error`, `ThrottlingException` |
| `Overloaded` | yes | Anthropic `overloaded_error`, `ServiceUnavailableException` |
| `Server` | yes | 5xx with no specific body |
| `Timeout` | yes | 408, body contains "timed out" |
| `Auth` | no | 401, 403, "invalid api key" |
| `ContextWindow` | no | `context_length_exceeded`, "maximum context length" |
| `ContentPolicy` | no | `content_filter`, "safety" |
| `Malformed` | no | 400 + `validation` / `bad request` |
| `NotFound` | no | 404, "model not found" |
| `BillingQuota` | no | 402, `insufficient_quota`, "billing" |
| `Unknown` | no | anything we don't recognize |

## Features

- `serde` — derive `Serialize`/`Deserialize` on `ErrorClass` (useful for
  logging).

## License

MIT or Apache-2.0.
