use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthProvider {
    Email,
    Google,
}

impl fmt::Display for AuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthProvider::Email => write!(f, "Email"),
            AuthProvider::Google => write!(f, "Google"),
        }
    }
}

impl std::str::FromStr for AuthProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Email" => Ok(AuthProvider::Email),
            "Google" => Ok(AuthProvider::Google),
            _ => Err(format!("'{}' is not a valid AuthProvider", s)),
        }
    }
}
