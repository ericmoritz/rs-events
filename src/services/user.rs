//!  This serves as the public API for the events service
use uuid::Uuid;
use diesel;
use std::fmt;
use models::user::{NewUser, User};
use models::user::IOModel;
use models::user::pg::PgModel;
use jsonwebtoken as jwt;
use std::default::Default;
use serde::ser::Serialize;

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

/// represents an OAuth 2.0 password grant
///
/// See: [rfc-6749 section-4.3.2](https://tools.ietf.org/html/rfc6749#section-4.3.2)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PasswordGrantRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
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

/// represents an OAuth 2.0 Refresh Token request
///
/// It is used to get a new access token if it expires
///
/// See: [RFC-6749 Section 6](https://tools.ietf.org/html/rfc6749#section-6)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct RefreshGrantRequest<'a> {
    /// The refresh token that was returned by the AccessTokenResponse
    pub refresh_token: &'a str,
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
pub struct RegisterRequest<'a> {
    pub name: &'a str,
    pub email: &'a str,
    /// The raw, unhashed password for the user
    pub password: &'a str,
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
pub struct ConfirmNewUserRequest<'a> {
    /// The confirm_token that was given out after regitering
    pub confirm_token: &'a str,
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
pub struct CurrentUserRequest<'a> {
    /// This is the OAuth 2.0 access token that authorizes the current user
    pub access_token: &'a str,
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

/// The API for the user service
pub struct Service<'a> {
    // TODO: make this generic so we can mock it out
    model: &'a PgModel<'a>,
    secret_key: &'a [u8],
}

impl<'a> Service<'a> {
    /// create a new Service instance
    pub fn new(model: &'a PgModel<'a>, secret_key: &'a [u8]) -> Service<'a> {
        Service { model, secret_key }
    }

    /// call to get an access token using a un/pw
    pub fn password_grant(
        &self,
        request: &PasswordGrantRequest,
    ) -> Result<AccessTokenResponse, ServiceError> {
        let user: User = self.model
            .verify_login(request.username, request.password)?
            .ok_or(ServiceError::PermissionDenied)?;

        Ok(access_token_response(self.secret_key, &user))
    }

    /// call to get a new access token using a refresh token
    pub fn refresh_token_grant(
        &self,
        request: &RefreshGrantRequest,
    ) -> Result<AccessTokenResponse, ServiceError> {
        let id = &validate_refresh_token(self.secret_key, request.refresh_token)
            .ok_or(ServiceError::PermissionDenied)?;

        let user = self.model.find(id)?.ok_or(ServiceError::PermissionDenied)?;

        Ok(access_token_response(self.secret_key, &user))
    }

    /// call to register a new user
    pub fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse, ServiceError> {
        let new_user = NewUser {
            id: &Uuid::new_v4(),
            name: request.name,
            password: request.password,
            email: request.email,
        };
        let user = self.model
            .create(&new_user)?
            .ok_or(ServiceError::UserExists)?;

        Ok(RegisterResponse {
            confirm_token: encode_token(
                self.secret_key,
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
        let id = &validate_confirm_token(self.secret_key, request.confirm_token)
            .ok_or(ServiceError::InvalidConfirmToken)?;

        self.model.confirm(id)?;

        Ok(ConfirmNewUserResponse)
    }

    /// get the user for a request token
    pub fn current_user(
        &self,
        request: &CurrentUserRequest,
    ) -> Result<CurrentUserResponse, ServiceError> {
        let id = &validate_access_token(self.secret_key, request.access_token)
            .ok_or(ServiceError::PermissionDenied)?;
        let user = self.model.find(id)?.ok_or(ServiceError::PermissionDenied)?;

        Ok(CurrentUserResponse {
            identifier: user.id,
            name: user.name,
            email: user.email,
        })
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
