/// Tests for query validation (ConfirmEmailQuery)
use auth_service::iam::identity::domain::model::queries::confirm_email_query::ConfirmEmailQuery;

#[test]
fn test_confirm_email_query_validation_success() {
    // Valid token (32+ characters)
    let token = "a".repeat(32);
    let query = ConfirmEmailQuery::new(token.clone());
    assert!(query.is_ok());
    assert_eq!(query.unwrap().token, token);
}

#[test]
fn test_confirm_email_query_validation_too_short() {
    // Token too short (less than 32 characters)
    let token = "short_token_123".to_string();
    let query = ConfirmEmailQuery::new(token);
    assert!(query.is_err());
}

#[test]
fn test_confirm_email_query_validation_exactly_32() {
    // Exactly 32 characters (boundary test)
    let token = "a".repeat(32);
    let query = ConfirmEmailQuery::new(token);
    assert!(query.is_ok());
}

#[test]
fn test_confirm_email_query_validation_31_chars_fails() {
    // 31 characters (just below limit)
    let token = "a".repeat(31);
    let query = ConfirmEmailQuery::new(token);
    assert!(query.is_err());
}

#[test]
fn test_confirm_email_query_validation_empty_fails() {
    // Empty token
    let token = String::new();
    let query = ConfirmEmailQuery::new(token);
    assert!(query.is_err());
}

#[test]
fn test_confirm_email_query_realistic_token() {
    // Realistic base64-encoded token
    let token = "K9j2mX_pqR8wT4vL3nZ5bH8mP2xQ7wY9A1B2C3D4E5F6G7H8".to_string();
    let query = ConfirmEmailQuery::new(token.clone());
    assert!(query.is_ok());
    assert_eq!(query.unwrap().token, token);
}

#[test]
fn test_confirm_email_query_with_special_chars() {
    // Token with URL-safe base64 characters
    let token = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij_-".to_string();
    let query = ConfirmEmailQuery::new(token.clone());
    assert!(query.is_ok());
}

#[test]
fn test_confirm_email_query_boundary_33_chars() {
    // 33 characters (just above limit)
    let token = "a".repeat(33);
    let query = ConfirmEmailQuery::new(token.clone());
    assert!(query.is_ok());
    assert_eq!(query.unwrap().token, token);
}

#[test]
fn test_confirm_email_query_very_long_token() {
    // Very long token (64 characters)
    let token = "a".repeat(64);
    let query = ConfirmEmailQuery::new(token.clone());
    assert!(query.is_ok());
    assert_eq!(query.unwrap().token, token);
}
