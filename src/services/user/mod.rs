// This serves as the public API for the events service
pub mod service;

use std::fmt;
use uuid::Uuid;


#[derive(Debug, Fail)]
pub enum ServiceError {
    InvalidConfirmToken,
    PermissionDenied,
    UserExists,
    Other,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// UserService is the API of the users service
pub trait UserService {
    // login is called to get an access token using a un/pw
    fn password_grant(&self, request: &PasswordGrantRequest) -> Result<AccessTokenResponse, ServiceError>;

    // refresh_token_grant is called to get a new access token
    fn refresh_token_grant(&self, request: &RefreshGrantRequest) -> Result<AccessTokenResponse, ServiceError>;

    // register is called when registering a new user
    fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse, ServiceError>;

    // confirm_new_user
    fn confirm_new_user(&self, request: &ConfirmNewUserRequest) -> Result<ConfirmNewUserResponse, ServiceError>;

    // Get the user for a request token
    fn current_user(&self, request: &CurrentUserRequest) -> Result<CurrentUserResponse, ServiceError>;
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
