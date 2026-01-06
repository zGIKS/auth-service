use crate::iam::identity::domain::error::DomainError;
use crate::iam::identity::domain::services::notification_service::NotificationService;
use crate::messaging::interfaces::acl::messaging_facade::MessagingFacade;
use async_trait::async_trait;

pub struct EmailService<F>
where
    F: MessagingFacade,
{
    messaging_facade: F,
}

impl<F> EmailService<F>
where
    F: MessagingFacade,
{
    pub fn new(messaging_facade: F) -> Self {
        Self { messaging_facade }
    }
}

#[async_trait]
impl<F> NotificationService for EmailService<F>
where
    F: MessagingFacade,
{
    async fn send_verification_email(
        &self,
        to: &str,
        verification_link: &str,
    ) -> Result<(), DomainError> {
        let subject = "Verify your account".to_string();
        let body = format!(
            "Please click the following link to verify your account: {}",
            verification_link
        );

        self.messaging_facade
            .send_email(to.to_string(), subject, body)
            .await
            .map_err(|e| DomainError::InternalError(format!("Failed to send email: {}", e)))
    }

    async fn send_password_reset_email(
        &self,
        to: &str,
        reset_link: &str,
    ) -> Result<(), DomainError> {
        let subject = "Password Reset Request".to_string();
        let body = format!(
            "You requested a password reset. Click the link below to set a new password:\n{}",
            reset_link
        );

        self.messaging_facade
            .send_email(to.to_string(), subject, body)
            .await
            .map_err(|e| DomainError::InternalError(format!("Failed to send email: {}", e)))
    }
}
