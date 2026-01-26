use crate::iam::identity::domain::model::{
    aggregates::identity::Identity, value_objects::email::Email,
};
use std::error::Error;
use std::future::Future;

#[cfg_attr(test, mockall::automock)]
pub trait IdentityRepository: Send + Sync {
    fn save(
        &self,
        identity: Identity,
    ) -> impl Future<Output = Result<Identity, Box<dyn Error + Send + Sync>>> + Send;
    fn find_by_email(
        &self,
        email: &Email,
    ) -> impl Future<Output = Result<Option<Identity>, Box<dyn Error + Send + Sync>>> + Send;
}
