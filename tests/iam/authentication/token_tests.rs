/// Tests for JWT token generation and validation
use auth_service::iam::authentication::domain::model::value_objects::token::Token;

#[test]
fn test_token_creation() {
    let token_value = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
    let token = Token::new(token_value.to_string());

    assert_eq!(token.value(), token_value);
}

#[test]
fn test_token_clone() {
    let token_value = "test_token_123".to_string();
    let token1 = Token::new(token_value.clone());
    let token2 = token1.clone();

    assert_eq!(token1.value(), token2.value());
}

#[test]
fn test_token_with_empty_string() {
    // Token should accept empty string (validation happens elsewhere)
    let token = Token::new(String::new());
    assert_eq!(token.value(), "");
}

#[test]
fn test_token_with_special_characters() {
    let token_value = "token_with-special.chars/+=".to_string();
    let token = Token::new(token_value.clone());
    assert_eq!(token.value(), token_value);
}

#[test]
fn test_token_immutability() {
    let original_value = "immutable_token".to_string();
    let token = Token::new(original_value.clone());

    // Token value should not change
    assert_eq!(token.value(), original_value);

    // Creating another token doesn't affect the first
    let _other_token = Token::new("different_token".to_string());
    assert_eq!(token.value(), original_value);
}

#[test]
fn test_realistic_jwt_token() {
    // Real JWT format
    let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI1ZjdhMjQ4MC1jOTI0LTRkYTctOGY5Yi0wZTNkNGE2YzQyYTEiLCJleHAiOjE3MDQ1MDQwMDAsImlhdCI6MTcwNDUwMDQwMH0.KjwJ8vH5n3pL9mR2tY6uI8oP4aQ1bS3cT7vU9wX0yZ1";
    let token = Token::new(jwt.to_string());

    assert_eq!(token.value(), jwt);
    assert!(token.value().contains("."));
    assert_eq!(token.value().split('.').count(), 3); // JWT has 3 parts
}

#[test]
fn test_token_long_value() {
    // Very long token
    let long_token = "a".repeat(500);
    let token = Token::new(long_token.clone());

    assert_eq!(token.value(), long_token);
    assert_eq!(token.value().len(), 500);
}
