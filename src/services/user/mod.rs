// This serves as the public API for the events service
pub mod service;

use std::fmt;
use uuid::Uuid;
use diesel;

/// ServiceError are errors that can happen with the service    
#[derive(Debug, Fail)]
pub enum ServiceError {
    InvalidConfirmToken,
    PermissionDenied,
    UserExists,
    DBError(diesel::result::Error),

}

impl From<diesel::result::Error> for ServiceError {
    fn from(it: diesel::result::Error) -> Self {
        ServiceError::DBError(it)
    }
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// https://tools.ietf.org/html/rfc6749#section-4.3
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PasswordGrantRequest<'a> {
    pub name: &'a str,
    pub password: &'a str,
}

// https://tools.ietf.org/html/rfc6749#section-4.1.4
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub token_type: String, // Allows "bearer"
    pub expires_in: i64,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaim {
    sub: String,
    access_token: bool,
}

// https://tools.ietf.org/html/rfc6749#section-6
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RefreshGrantRequest<'a> {
    pub refresh_token: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaim {
    sub: String,
    refresh_token: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequest<'a> {
    pub name: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    pub confirm_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfirmNewUserRequest<'a> {
    pub confirm_token: &'a str,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfirmNewUserResponse;

#[derive(Debug, Default, Serialize, Deserialize)]
struct ConfirmTokenClaim {
    sub: String,
    confirm_token: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentUserRequest<'a> {
    pub access_token: &'a str,
}

// https://schema.org/Person
#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentUserResponse {
    // https://schema.org/Thing
    pub identifier: Uuid,
    pub name: String,

    // https://schema.org/Person
    pub email: String,
}
