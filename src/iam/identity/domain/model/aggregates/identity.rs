use crate::iam::identity::domain::model::value_objects::{
    auth_provider::AuthProvider, email::Email, identity_id::IdentityId, password::Password,
};
use crate::iam::identity::domain::model::commands::register_identity_command::RegisterIdentityCommand;
use crate::shared::domain::model::entities::auditable_model::AuditableModel;

#[derive(Debug, Clone)]
pub struct Identity {
    id: IdentityId,
    email: Email,
    password: Password,
    provider: AuthProvider,
    is_verified: bool,
    audit: AuditableModel,
}

impl Identity {
    pub fn new(
        id: IdentityId,
        email: Email,
        password: Password,
        provider: AuthProvider,
        is_verified: bool,
        audit: AuditableModel,
    ) -> Self {
        Self {
            id,
            email,
            password,
            provider,
            is_verified,
            audit,
        }
    }

    pub fn register(command: RegisterIdentityCommand) -> Self {
        Self {
            id: IdentityId::new(),
            email: command.email,
            password: command.password,
            provider: command.provider,
            is_verified: command.is_verified,
            audit: AuditableModel::new(),
        }
    }

    pub fn id(&self) -> &IdentityId {
        &self.id
    }

    pub fn email(&self) -> &Email {
        &self.email
    }

    pub fn password(&self) -> &Password {
        &self.password
    }

    pub fn provider(&self) -> &AuthProvider {
        &self.provider
    }

    pub fn is_verified(&self) -> bool {
        self.is_verified
    }

    pub fn audit(&self) -> &AuditableModel {
        &self.audit
    }
}