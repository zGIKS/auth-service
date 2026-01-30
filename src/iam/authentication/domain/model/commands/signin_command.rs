use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct SigninCommand {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
    pub ip_address: Option<String>,
}

impl SigninCommand {
    pub fn new(email: String, password: String, ip_address: Option<String>) -> Self {
        Self {
            email,
            password,
            ip_address,
        }
    }
}
