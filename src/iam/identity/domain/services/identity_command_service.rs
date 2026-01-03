use crate::iam::identity::domain::model::{
    aggregates::identity::Identity,
    commands::register_identity_command::RegisterIdentityCommand,
};
use crate::iam::identity::domain::error::DomainError;
use std::future::Future;

pub trait IdentityCommandService: Send + Sync {
    fn handle(
        &self,
        command: RegisterIdentityCommand,
    ) -> impl Future<Output = Result<Identity, DomainError>> + Send;
}