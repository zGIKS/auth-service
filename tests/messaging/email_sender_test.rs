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

    // Prioritize TEST_SMTP_ variables, fallback to standard SMTP_ if not present
    let host = std::env::var("TEST_SMTP_HOST").unwrap_or_else(|_| std::env::var("SMTP_HOST").unwrap_or_default());
    let port = std::env::var("TEST_SMTP_PORT").unwrap_or_else(|_| std::env::var("SMTP_PORT").unwrap_or_default());
    let username = std::env::var("TEST_SMTP_USERNAME").unwrap_or_else(|_| std::env::var("SMTP_USERNAME").unwrap_or_default());
    let password = match std::env::var("TEST_SMTP_PASSWORD") {
        Ok(v) => v,
        Err(_) => match std::env::var("SMTP_PASSWORD") {
            Ok(v) => v,
            Err(_) => {
                println!("Skipping email test: TEST_SMTP_PASSWORD or SMTP_PASSWORD not set");
                return;
            }
        },
    };
    let from = std::env::var("TEST_SMTP_FROM").unwrap_or_else(|_| std::env::var("SMTP_FROM").unwrap_or_default());
    
    // For Resend, we must use a verified email or their official test addresses.
    let to_addr = match std::env::var("TEST_SMTP_TO") {
        Ok(v) => v,
        Err(_) => {
            println!("Skipping email test: TEST_SMTP_TO not set");
            return;
        }
    };

    // Override environment variables so SmtpEmailSender::new() picks them up
    unsafe {
        std::env::set_var("SMTP_HOST", host);
        std::env::set_var("SMTP_PORT", port);
        std::env::set_var("SMTP_USERNAME", username);
        std::env::set_var("SMTP_PASSWORD", password);
        std::env::set_var("SMTP_FROM", from);
    }

    let sender =
        SmtpEmailSender::new(create_circuit_breaker()).expect("Failed to create SMTP sender");

    let to = match EmailAddress::new(to_addr.clone()) {
        Ok(email) => email,
        Err(_) => {
            println!("Skipping email test: target recipient is not a valid email ({})", to_addr);
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
