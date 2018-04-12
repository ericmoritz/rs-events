//!  This serves as the public API for the events service
use actix::prelude::*;
use actix_web::*;
use diesel;
use jsonwebtoken as jwt;
use models::user::pg::PgModel;
use models::user::IOModel;
use models::user::{NewUser, User};
use serde::ser::Serialize;
use std::default::Default;
use std::fmt;
use uuid::Uuid;

/// The API for the user service
pub struct Service {
    // TODO: make this generic so we can mock it out
    model: PgModel,
    secret_key: Vec<u8>,
}

impl Actor for Service {
    type Context = SyncContext<Self>;
}

impl Service {
    /// create a new Service instance
    pub fn new(model: PgModel, secret_key: Vec<u8>) -> Service {
        Service { model, secret_key }
    }

    /// call to get a new access token using a TokenRequest
    pub fn token(&self, request: &TokenRequest) -> Result<AccessTokenResponse, ServiceError> {
        let user: User = match *request {
            TokenRequest::RefreshToken { ref refresh_token } => {
                let id = &validate_refresh_token(&self.secret_key, refresh_token)
                    .ok_or(ServiceError::PermissionDenied)?;
                self.model.find(id)?.ok_or(ServiceError::PermissionDenied)?
            }
            TokenRequest::Password {
                ref username,
                ref password,
            } => self.model
                .verify_login(username, password)?
                .ok_or(ServiceError::PermissionDenied)?,
        };
        Ok(access_token_response(&self.secret_key, &user))
    }

    /// call to register a new user
    pub fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse, ServiceError> {
        let new_user = &NewUser {
            id: &Uuid::new_v4(),
            name: &request.name,
            password: &request.password,
            email: &request.email,
        };
        let user = self.model
            .create(new_user)?
            .ok_or(ServiceError::UserExists)?;

        Ok(RegisterResponse {
            confirm_token: encode_token(
                &self.secret_key,
                ConfirmTokenClaim {
                    sub: user.id.simple().to_string(),
                    confirm_token: true,
                },
            ),
        })
    }

    /// confirm a user
    pub fn confirm_new_user(
        &self,
        request: &ConfirmNewUserRequest,
    ) -> Result<ConfirmNewUserResponse, ServiceError> {
        let id = &validate_confirm_token(&self.secret_key, &request.confirm_token)
            .ok_or(ServiceError::InvalidConfirmToken)?;

        self.model.confirm(id)?;

        Ok(ConfirmNewUserResponse)
    }

    /// get the user for a request token
    pub fn current_user(
        &self,
        request: &CurrentUserRequest,
    ) -> Result<CurrentUserResponse, ServiceError> {
        let id = &validate_access_token(&self.secret_key, &request.access_token)
            .ok_or(ServiceError::PermissionDenied)?;
        let user = self.model.find(id)?.ok_or(ServiceError::PermissionDenied)?;

        Ok(CurrentUserResponse {
            identifier: user.id,
            name: user.name,
            email: user.email,
        })
    }
}

/// errors that can happen with the service
///
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

/// represents an OAuth 2.0 password or refresh_token grant
///
/// See: [rfc-6749 section-4.3.2](https://tools.ietf.org/html/rfc6749#section-4.3.2)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "grant_type")]
pub enum TokenRequest {
    #[serde(rename = "password")]
    Password { username: String, password: String },
    #[serde(rename = "refresh_token")]
    RefreshToken { refresh_token: String },
}

impl Message for TokenRequest {
    type Result = Result<AccessTokenResponse, ServiceError>;
}

impl Handler<TokenRequest> for Service {
    type Result = Result<AccessTokenResponse, ServiceError>;

    fn handle(&mut self, request: TokenRequest, _: &mut Self::Context) -> Self::Result {
        self.token(&request)
    }
}

/// represents an OAuth 2.0 Access Token Response
///
/// See: [rfc-6749 section 4.1.4](https://tools.ietf.org/html/rfc6749#section-4.1.4)
#[derive(Default, Serialize, Deserialize, Debug)]
pub struct AccessTokenResponse {
    /// This is the OAuth 2.0 access token that is used when making request on behalf of a user
    pub access_token: String,
    /// This is part of the OAuth 2.0 spec and is always "bearer"
    pub token_type: String,
    /// Number of seconds until the token expires
    pub expires_in: i64,
    /// The token used to refresh the access token when it expires
    pub refresh_token: String,
}

/// represents the data inside of the JWT for the access token
///
#[derive(Debug, Serialize, Deserialize)]
struct AccessTokenClaim {
    /// The standard JWT subject field
    sub: String,
    /// A flag that marks the JSON as an access token so that the access token is shaped differntly that
    /// a refresh or confirm token.  Without this flag, a refresh or confirm token could be used as an access token
    access_token: bool,
}

/// represents the data inside of the [JWT](https://en.wikipedia.org/wiki/JSON_Web_Token) for the refresh token
///
#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenClaim {
    /// The standard JWT subject field
    sub: String,
    /// The flag that makes the claim data a refresh token. See the explanation in [`AccessTokenClaim`]
    refresh_token: bool,
}

/// represents the form that is needed to register a new user
///
/// It is formatted as a [schema:Person](https://schema.org/Person) with an additional
/// `password` field.
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    /// The raw, unhashed password for the user
    pub password: String,
}

impl Message for RegisterRequest {
    type Result = Result<RegisterResponse, ServiceError>;
}

impl Handler<RegisterRequest> for Service {
    type Result = Result<RegisterResponse, ServiceError>;

    fn handle(&mut self, request: RegisterRequest, _: &mut Self::Context) -> Self::Result {
        self.register(&request)
    }
}

/// contains the confirmation token for confirming the new user
///
#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    /// This is used in the ConfirmNewUserRequest in order to activate a new user
    pub confirm_token: String,
}

/// used to confirm the new user
///
/// The `confirm_token` is the same confirm token that was given out after registering

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfirmNewUserRequest {
    /// The confirm_token that was given out after regitering
    pub confirm_token: String,
}

impl Message for ConfirmNewUserRequest {
    type Result = Result<ConfirmNewUserResponse, ServiceError>;
}

impl Handler<ConfirmNewUserRequest> for Service {
    type Result = Result<ConfirmNewUserResponse, ServiceError>;

    fn handle(&mut self, request: ConfirmNewUserRequest, _: &mut Self::Context) -> Self::Result {
        self.confirm_new_user(&request)
    }
}

/// the response from a new user request
///
/// This is currently an empty object but may be filled in later
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfirmNewUserResponse;

/// the data inside the JWT for the confirmation token
///
#[derive(Debug, Default, Serialize, Deserialize)]
struct ConfirmTokenClaim {
    sub: String,
    confirm_token: bool,
}

/// used to get the data about the user that has this access token
///
#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentUserRequest {
    /// This is the OAuth 2.0 access token that authorizes the current user
    pub access_token: String,
}

impl Message for CurrentUserRequest {
    type Result = Result<CurrentUserResponse, ServiceError>;
}

impl Handler<CurrentUserRequest> for Service {
    type Result = Result<CurrentUserResponse, ServiceError>;

    fn handle(&mut self, request: CurrentUserRequest, _: &mut Self::Context) -> Self::Result {
        self.current_user(&request)
    }
}

/// the data about the user
///
/// It is formatted as a [schema:Person](https://schema.org/Person)
///
#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentUserResponse {
    // https://schema.org/Thing
    pub identifier: Uuid,
    pub name: String,

    // https://schema.org/Person
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusResponse {
    pub status: &'static str,
}

impl Message for StatusRequest {
    type Result = Result<StatusResponse, ServiceError>;
}

impl Handler<StatusRequest> for Service {
    type Result = Result<StatusResponse, ServiceError>;

    fn handle(&mut self, _: StatusRequest, _: &mut Self::Context) -> Self::Result {
        Ok(StatusResponse { status: "up" })
    }
}

// Internal

fn validate_confirm_token(key: &[u8], token: &str) -> Option<Uuid> {
    let token_result = jwt::decode::<ConfirmTokenClaim>(token, key, &jwt::Validation::default());

    token_result.ok().and_then(|token| {
        if token.claims.confirm_token {
            Uuid::parse_str(&token.claims.sub).ok()
        } else {
            None
        }
    })
}

fn validate_access_token(key: &[u8], token: &str) -> Option<Uuid> {
    if let Ok(data) = jwt::decode::<AccessTokenClaim>(token, key, &jwt::Validation::default()) {
        if data.claims.access_token {
            return Uuid::parse_str(&data.claims.sub).ok();
        }
    }
    None
}

fn validate_refresh_token(key: &[u8], token: &str) -> Option<Uuid> {
    if let Ok(data) = jwt::decode::<RefreshTokenClaim>(token, key, &jwt::Validation::default()) {
        if data.claims.refresh_token {
            return Uuid::parse_str(&data.claims.sub).ok();
        }
    }
    None
}

fn encode_token<T: Serialize>(key: &[u8], claims: T) -> String {
    // TODO: handle error correctly
    jwt::encode(&jwt::Header::default(), &claims, key).unwrap_or_else(|_| "".into())
}

fn access_token_response(key: &[u8], user: &User) -> AccessTokenResponse {
    AccessTokenResponse {
        access_token: encode_token(
            key,
            AccessTokenClaim {
                sub: user.id.simple().to_string(),
                access_token: true,
            },
        ),
        refresh_token: encode_token(
            key,
            RefreshTokenClaim {
                sub: user.id.simple().to_string(),
                refresh_token: true,
            },
        ),
        token_type: "bearer".into(),
        expires_in: 3600,
    }
}
