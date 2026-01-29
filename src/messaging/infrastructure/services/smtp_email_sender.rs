use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    transport::smtp::authentication::Credentials,
};
use std::env;

use crate::messaging::domain::{
    error::MessagingError,
    model::value_objects::{body::Body, email_address::EmailAddress, subject::Subject},
    services::email_sender_service::EmailSenderService,
};
use crate::shared::infrastructure::circuit_breaker::AppCircuitBreaker;

pub struct SmtpEmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
    circuit_breaker: AppCircuitBreaker,
}

impl SmtpEmailSender {
    pub fn new(circuit_breaker: AppCircuitBreaker) -> Result<Self, MessagingError> {
        let host = env::var("SMTP_HOST")
            .map_err(|_| MessagingError::ConfigError("SMTP_HOST not set".to_string()))?;
        let username = env::var("SMTP_USERNAME")
            .map_err(|_| MessagingError::ConfigError("SMTP_USERNAME not set".to_string()))?;
        let password = env::var("SMTP_PASSWORD")
            .map_err(|_| MessagingError::ConfigError("SMTP_PASSWORD not set".to_string()))?;

        let port_str = env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string());
        let port = port_str
            .parse::<u16>()
            .map_err(|_| MessagingError::ConfigError("SMTP_PORT invalid".to_string()))?;

        let credentials = Credentials::new(username.clone(), password);

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
            .map_err(|e| MessagingError::ConfigError(format!("Failed to build transport: {}", e)))?
            .port(port)
            .credentials(credentials)
            .build();

        Ok(Self {
            mailer,
            from: username,
            circuit_breaker,
        })
    }
}

#[async_trait]
impl EmailSenderService for SmtpEmailSender {
    async fn send(
        &self,
        to: &EmailAddress,
        subject: &Subject,
        body: &Body,
    ) -> Result<(), MessagingError> {
        if !self.circuit_breaker.is_call_permitted().await {
            return Err(MessagingError::SendError(
                "Circuit breaker open".to_string(),
            ));
        }

        let result = async {
            let email = Message::builder()
                .from(
                    self.from.parse().map_err(|_| {
                        MessagingError::ConfigError("Invalid FROM address".to_string())
                    })?,
                )
                .to(to
                    .value()
                    .parse()
                    .map_err(|_| MessagingError::SendError("Invalid TO address".to_string()))?)
                .subject(subject.value())
                .body(body.value().to_string())
                .map_err(|e| MessagingError::SendError(format!("Failed to build email: {}", e)))?;

            self.mailer
                .send(email)
                .await
                .map_err(|e| MessagingError::SendError(format!("Failed to send email: {}", e)))
        }
        .await;

        match result {
            Ok(_) => {
                self.circuit_breaker.on_success().await;
                Ok(())
            }
            Err(e) => {
                self.circuit_breaker.on_failure().await;
                Err(e)
            }
        }
    }
}
