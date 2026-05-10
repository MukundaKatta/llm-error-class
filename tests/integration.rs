use llm_error_class::{classify, ErrorClass};

#[test]
fn anthropic_rate_limit() {
    let body = r#"{"type":"error","error":{"type":"rate_limit_error","message":"slow down"}}"#;
    assert_eq!(classify(429, body), ErrorClass::RateLimit);
}

#[test]
fn anthropic_overloaded() {
    let body = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;
    assert_eq!(classify(529, body), ErrorClass::Overloaded);
}

#[test]
fn openai_rate_limit() {
    let body = r#"{"error":{"type":"rate_limit_error","code":"rate_limit_exceeded"}}"#;
    assert_eq!(classify(429, body), ErrorClass::RateLimit);
}

#[test]
fn openai_context_window() {
    let body = r#"{"error":{"code":"context_length_exceeded","message":"This model's maximum context length is 200000 tokens"}}"#;
    assert_eq!(classify(400, body), ErrorClass::ContextWindow);
}

#[test]
fn openai_insufficient_quota() {
    let body = r#"{"error":{"code":"insufficient_quota","message":"You exceeded your current quota"}}"#;
    assert_eq!(classify(429, body), ErrorClass::BillingQuota);
}

#[test]
fn bedrock_throttling() {
    let body = r#"{"__type":"ThrottlingException","message":"Rate exceeded"}"#;
    assert_eq!(classify(400, body), ErrorClass::RateLimit);
}

#[test]
fn bedrock_validation_is_malformed() {
    let body = r#"{"__type":"ValidationException","message":"Input is malformed"}"#;
    assert_eq!(classify(400, body), ErrorClass::Malformed);
}

#[test]
fn bedrock_service_unavailable() {
    let body = r#"{"__type":"ServiceUnavailableException"}"#;
    assert_eq!(classify(503, body), ErrorClass::Overloaded);
}

#[test]
fn content_policy_block() {
    let body = r#"{"error":{"code":"content_filter","message":"Response blocked by safety filter"}}"#;
    assert_eq!(classify(400, body), ErrorClass::ContentPolicy);
}

#[test]
fn auth_failure() {
    let body = r#"{"error":{"message":"Invalid API key"}}"#;
    assert_eq!(classify(401, body), ErrorClass::Auth);
}

#[test]
fn falls_back_to_status_code() {
    // No body keywords; just status.
    assert_eq!(classify(500, "unknown"), ErrorClass::Server);
    assert_eq!(classify(404, ""), ErrorClass::NotFound);
    assert_eq!(classify(403, "{}"), ErrorClass::Auth);
}

#[test]
fn retriable_classes() {
    assert!(ErrorClass::RateLimit.is_retriable());
    assert!(ErrorClass::Overloaded.is_retriable());
    assert!(ErrorClass::Server.is_retriable());
    assert!(ErrorClass::Timeout.is_retriable());

    assert!(!ErrorClass::Auth.is_retriable());
    assert!(!ErrorClass::ContextWindow.is_retriable());
    assert!(!ErrorClass::ContentPolicy.is_retriable());
    assert!(!ErrorClass::Malformed.is_retriable());
    assert!(!ErrorClass::NotFound.is_retriable());
    assert!(!ErrorClass::BillingQuota.is_retriable());
    assert!(!ErrorClass::Unknown.is_retriable());
}

#[test]
fn empty_body_unknown_status() {
    assert_eq!(classify(418, ""), ErrorClass::Unknown);
}
