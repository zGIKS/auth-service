use hickory_resolver::TokioAsyncResolver;
use serde::{Deserialize, Serialize};
use std::error::Error;
use validator::Validate;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct Email {
    #[validate(email, length(max = 254))]
    value: String,
}

impl Email {
    pub fn new(value: String) -> Result<Self, validator::ValidationErrors> {
        let email = Self { value };
        email.validate()?;
        Ok(email)
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub async fn validate_mx(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let parts: Vec<&str> = self.value.split('@').collect();
        if parts.len() != 2 {
            return Err("Invalid email format".into());
        }
        let domain = parts[1];

        let resolver = TokioAsyncResolver::tokio_from_system_conf()?;
        let mx_response = resolver.mx_lookup(domain).await;

        match mx_response {
            Ok(records) => {
                if records.iter().count() > 0 {
                    Ok(())
                } else {
                    Err(format!("No MX records found for domain: {}", domain).into())
                }
            }
            Err(_) => Err(format!("Failed to lookup MX records for domain: {}", domain).into()),
        }
    }
}
