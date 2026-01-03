use crate::iam::identity::domain::model::{
    aggregates::identity::Identity,
    commands::register_identity_command::RegisterIdentityCommand,
};
use std::error::Error;
use std::future::Future;

pub trait IdentityCommandService: Send + Sync {
    fn handle(
        &self,
        command: RegisterIdentityCommand,
    ) -> impl Future<Output = Result<Identity, Box<dyn Error + Send + Sync>>> + Send;
}