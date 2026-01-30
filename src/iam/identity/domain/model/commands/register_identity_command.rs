use crate::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, email::Email, password::Password,
};

#[derive(Debug, Clone)]
pub struct RegisterIdentityCommand {
    pub email: Email,
    pub password: Password,
    pub provider: AuthProvider,
}

impl RegisterIdentityCommand {
    pub fn new(email: Email, password: Password, provider: AuthProvider) -> Self {
        Self {
            email,
            password,
            provider,
        }
    }
}
