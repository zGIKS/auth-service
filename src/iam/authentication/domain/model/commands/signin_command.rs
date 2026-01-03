use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct SigninCommand {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}

impl SigninCommand {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}
