use crate::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, email::Email, password::Password,
};

#[derive(Debug, Clone)]
pub struct RegisterIdentityCommand {
    pub email: Email,
    pub password: Password,
    pub provider: AuthProvider,
    pub is_verified: bool,
}

impl RegisterIdentityCommand {
    pub fn new(
        email: Email,
        password: Password,
        provider: AuthProvider,
        is_verified: bool,
    ) -> Self {
        Self {
            email,
            password,
            provider,
            is_verified,
        }
    }
}
