/// Authentication bounded context tests
/// Organized by feature/use case for better maintainability

// Shared test utilities
pub mod test_mocks;

// Feature-specific tests
mod signin_command_tests;
mod token_tests;
mod integration_tests;
mod refresh_token_tests;

// Legacy test files (keep for backwards compatibility)
pub mod signin_tests;
pub mod infrastructure_tests;