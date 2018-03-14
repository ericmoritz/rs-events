// This serves as the public API for the events service
pub mod service;

use failure::Error;
use std::fmt;
use uuid::Uuid;

use actix::prelude::*;


#[derive(Debug, Fail)]
pub enum ServiceError {
    InvalidConfirmToken,
    PermissionDenied,
    UserExists
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// UserService is the API of the users service
pub trait UserService {
    // login is called to get an access token using a un/pw
    fn password_grant(&self, request: &PasswordGrantRequest) -> Result<AccessTokenResponse, Error>;

    // refresh_token_grant is called to get a new access token
    fn refresh_token_grant(&self, request: &RefreshGrantRequest) -> Result<AccessTokenResponse, Error>;

    // register is called when registering a new user
    fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse, Error>;

    // confirm_new_user
    fn confirm_new_user(&self, request: &ConfirmNewUserRequest) -> Result<ConfirmNewUserResponse, Error>;

    // Get the user for a request token
    fn current_user(&self, request: &CurrentUserRequest) -> Result<CurrentUserResponse, Error>;
}

// https://tools.ietf.org/html/rfc6749#section-4.2.2
pub type AccessToken = String;
// https://tools.ietf.org/html/rfc6749#section-4.2.2
pub type RefreshToken = String;
// Probably a [JWT](https://tools.ietf.org/html/rfc7519) for confirming the email
pub type ConfirmToken = String;

// https://tools.ietf.org/html/rfc6749#section-4.3
#[derive(Serialize, Deserialize, Debug)]
pub struct PasswordGrantRequest {
    pub name: String,
    pub password: String,
    pub client_id: String,
}
impl Message for PasswordGrantRequest {
    type Result = Result<AccessTokenResponse, Error>;
}

// https://tools.ietf.org/html/rfc6749#section-4.1.4
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct AccessTokenResponse {
    pub access_token: AccessToken,
    pub token_type: String, // Allows "bearer"
    pub expires_in: i64,
    pub refresh_token: RefreshToken,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaim {
    sub: String,
    access_token: bool,
}

// https://tools.ietf.org/html/rfc6749#section-6
#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshGrantRequest {
    pub refresh_token: RefreshToken,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaim {
    sub: String,
    refresh_token: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}
impl Message for RegisterRequest {
    type Result = Result<RegisterResponse, Error>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    pub confirm_token: ConfirmToken,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfirmNewUserRequest {
    pub confirm_token: ConfirmToken,
}
impl Message for ConfirmNewUserRequest {
    type Result = Result<ConfirmNewUserResponse, Error>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfirmNewUserResponse;

#[derive(Debug, Default, Serialize, Deserialize)]
struct ConfirmTokenClaim {
    sub: String,
    confirm_token: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentUserRequest {
    pub access_token: AccessToken,
}
impl Message for CurrentUserRequest {
    type Result = Result<CurrentUserResponse, Error>;
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
