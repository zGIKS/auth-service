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

    // Resend uses SMTP_USERNAME=resend (not an email), so target recipient must come from SMTP_TO.
    let to_addr = match std::env::var("SMTP_TO") {
        Ok(value) => value,
        Err(_) => {
            println!("Skipping email test: SMTP_TO not set");
            return;
        }
    };

    let to = match EmailAddress::new(to_addr.clone()) {
        Ok(email) => email,
        Err(_) => {
            println!("Skipping email test: SMTP_TO is not a valid email ({})", to_addr);
            return;
        }
    };
    let subject = Subject::new("Integration Test Email".to_string()).unwrap();
    let body =
        Body::new("This is a test email from the auth-service integration test.".to_string())
            .unwrap();

    let result = sender.send(&to, &subject, &body).await;

    match result {
        Ok(_) => println!("Email sent successfully to {}", to_addr),
        Err(e) => {
            let err_text = format!("{:?}", e);
            if err_text.contains("You can only send testing emails to your own email address") {
                println!("Skipping email test: Resend sandbox recipient restriction ({})", err_text);
                return;
            }
            panic!("Failed to send email: {:?}", e);
        }
    }
}
