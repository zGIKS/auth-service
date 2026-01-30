/// Authentication bounded context tests
/// Organized by feature/use case for better maintainability
// Shared test utilities
pub mod test_mocks;

// Feature-specific tests
mod integration_tests;
mod refresh_token_tests;
mod signin_command_tests;
mod token_tests;

// Legacy test files (keep for backwards compatibility)
pub mod infrastructure_tests;
pub mod lockout_tests;
pub mod signin_tests;
