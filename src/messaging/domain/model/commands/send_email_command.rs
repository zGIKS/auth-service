use crate::messaging::domain::model::value_objects::{
    body::Body, email_address::EmailAddress, subject::Subject,
};
use validator::ValidationErrors;

#[derive(Debug, Clone)]
pub struct SendEmailCommand {
    pub to: EmailAddress,
    pub subject: Subject,
    pub body: Body,
}

impl SendEmailCommand {
    pub fn new(to: String, subject: String, body: String) -> Result<Self, ValidationErrors> {
        Ok(Self {
            to: EmailAddress::new(to)?,
            subject: Subject::new(subject)?,
            body: Body::new(body)?,
        })
    }
}
