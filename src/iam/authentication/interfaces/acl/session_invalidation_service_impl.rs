use crate::iam::authentication::domain::services::authentication_command_service::SessionRepository;
use crate::iam::identity::domain::services::session_invalidation_service::SessionInvalidationService;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

pub struct SessionInvalidationServiceImpl<S>
where
    S: SessionRepository,
{
    session_repository: S,
}

impl<S> SessionInvalidationServiceImpl<S>
where
    S: SessionRepository,
{
    pub fn new(session_repository: S) -> Self {
        Self { session_repository }
    }
}

#[async_trait]
impl<S> SessionInvalidationService for SessionInvalidationServiceImpl<S>
where
    S: SessionRepository,
{
    async fn invalidate_all_sessions(
        &self,
        user_id: Uuid,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.session_repository
            .revoke_all_user_sessions(user_id)
            .await
    }
}
