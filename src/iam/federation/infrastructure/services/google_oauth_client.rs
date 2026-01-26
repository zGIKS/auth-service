use async_trait::async_trait;
use serde::Deserialize;

use crate::iam::federation::domain::{
    error::FederationError, model::value_objects::google_user::GoogleUser,
    services::google_oauth_service::GoogleOAuthService,
};

#[derive(Debug, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_in: Option<i64>,
    #[allow(dead_code)]
    token_type: Option<String>,
    #[allow(dead_code)]
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfoResponse {
    sub: String,
    email: String,
    email_verified: Option<bool>,
    name: Option<String>,
    picture: Option<String>,
}

pub struct GoogleOAuthClient {
    client: reqwest::Client,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl GoogleOAuthClient {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            client_id,
            client_secret,
            redirect_uri,
        }
    }
}

#[async_trait]
impl GoogleOAuthService for GoogleOAuthClient {
    async fn exchange_code(&self, code: String) -> Result<GoogleUser, FederationError> {
        let params = [
            ("code", code),
            ("client_id", self.client_id.clone()),
            ("client_secret", self.client_secret.clone()),
            ("redirect_uri", self.redirect_uri.clone()),
            ("grant_type", "authorization_code".to_string()),
        ];

        let token_response = self
            .client
            .post("https://oauth2.googleapis.com/token")
            .form(&params)
            .send()
            .await
            .map_err(|e| FederationError::TokenExchange(e.to_string()))?;

        if !token_response.status().is_success() {
            let status = token_response.status();
            let body = token_response.text().await.unwrap_or_default();
            return Err(FederationError::TokenExchange(format!(
                "status: {status}, body: {body}"
            )));
        }

        let tokens: GoogleTokenResponse = token_response
            .json()
            .await
            .map_err(|e| FederationError::TokenExchange(e.to_string()))?;

        let userinfo_response = self
            .client
            .get("https://www.googleapis.com/oauth2/v3/userinfo")
            .bearer_auth(&tokens.access_token)
            .send()
            .await
            .map_err(|e| FederationError::UserInfo(e.to_string()))?;

        if !userinfo_response.status().is_success() {
            let status = userinfo_response.status();
            let body = userinfo_response.text().await.unwrap_or_default();
            return Err(FederationError::UserInfo(format!(
                "status: {status}, body: {body}"
            )));
        }

        let userinfo: GoogleUserInfoResponse = userinfo_response
            .json()
            .await
            .map_err(|e| FederationError::UserInfo(e.to_string()))?;

        Ok(GoogleUser {
            sub: userinfo.sub,
            email: userinfo.email,
            email_verified: userinfo.email_verified.unwrap_or(false),
            name: userinfo.name,
            picture: userinfo.picture,
        })
    }
}
