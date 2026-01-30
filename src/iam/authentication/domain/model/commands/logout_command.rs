use validator::Validate;

#[derive(Debug, Validate)]
pub struct LogoutCommand {
    pub refresh_token: String,
}

impl LogoutCommand {
    pub fn new(refresh_token: String) -> Self {
        Self { refresh_token }
    }
}
