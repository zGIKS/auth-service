/// Identity bounded context tests
/// Organized by feature/use case for better maintainability

// Shared test utilities
mod test_mocks;

// Feature-specific tests
mod registration_command_tests;
mod email_confirmation_tests;
mod query_validation_tests;
mod password_reset_request_tests;
mod password_reset_execution_tests;

// Keep old file for backwards compatibility during transition
pub mod registration_tests;

