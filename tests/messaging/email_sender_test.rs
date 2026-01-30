use auth_service::messaging::domain::model::value_objects::{
    body::Body, email_address::EmailAddress, subject::Subject,
};
use auth_service::messaging::domain::services::email_sender_service::EmailSenderService;
use auth_service::messaging::infrastructure::services::smtp_email_sender::SmtpEmailSender;
use auth_service::shared::infrastructure::circuit_breaker::create_circuit_breaker;
use dotenvy::dotenv;

#[tokio::test]
async fn test_send_email_integration() {
    dotenv().ok();

    // Skip test if SMTP config is missing (so CI doesn't fail without creds)
    if std::env::var("SMTP_PASSWORD").is_err() {
        println!("Skipping email test: SMTP_PASSWORD not set");
        return;
    }

    let sender =
        SmtpEmailSender::new(create_circuit_breaker()).expect("Failed to create SMTP sender");

    // Replace with a valid email to test, or use the configured user email
    let to_addr = std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "test@example.com".to_string());

    let to = EmailAddress::new(to_addr.clone()).unwrap();
    let subject = Subject::new("Integration Test Email".to_string()).unwrap();
    let body =
        Body::new("This is a test email from the auth-service integration test.".to_string())
            .unwrap();

    let result = sender.send(&to, &subject, &body).await;

    match result {
        Ok(_) => println!("Email sent successfully to {}", to_addr),
        Err(e) => panic!("Failed to send email: {:?}", e),
    }
}
