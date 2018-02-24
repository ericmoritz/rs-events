
use uuid::Uuid;
use super::*;
use diesel::pg::PgConnection;
use models::user::{NewUser, User};
use jsonwebtoken as jwt;
use std::default::Default;
use failure::Error;
use serde::ser::Serialize;

pub struct Service<'a> {
    conn: &'a PgConnection,
    secret_key: &'a [u8],
}

impl<'a> Service<'a> {
    pub fn new(conn: &'a PgConnection, secret_key: &'a [u8]) -> Service<'a> {
        Service{conn, secret_key}
    }
}

impl<'a> UserService for Service<'a> {
   // login is called to get an access token using a un/pw
    fn password_grant(&self, request: &PasswordGrantRequest) -> Result<AccessTokenResponse, Error> {
        if let Some(user) = User::login(self.conn, &request.name, &request.password)? {
            return Ok(access_token_response(self.secret_key, user))
        }
        Err(ServiceError::PermissionDenied.into())
    }

    // refresh_token_grant is called to get a new access token
    fn refresh_token_grant(&self, request: &RefreshGrantRequest) -> Result<AccessTokenResponse, Error> {
        if let Some(id) = validate_refresh_token(self.secret_key, &request.refresh_token) {
            if let Some(user) = User::find(self.conn, id)? {
                return Ok(access_token_response(self.secret_key, user))
            }
        }
        Err(ServiceError::PermissionDenied.into())
    }

    // register is called when registering a new user
    fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse, Error> {
        let user = NewUser{
            id: Uuid::new_v4(),
            name: request.name.clone(),
            password: request.password.clone(),
            email: request.email.clone(),
        }.create(self.conn)?;

        match user {
            None => Err(ServiceError::UserExists.into()),
            Some(user) => Ok(RegisterResponse{
                confirm_token: encode_token(self.secret_key, 
                    ConfirmTokenClaim{sub: user.id.simple().to_string(), confirm_token: true}
                )
            })
        }
    }

    // confirm_new_user
    fn confirm_new_user(&self, request: &ConfirmNewUserRequest) -> Result<ConfirmNewUserResponse, Error> {
        match validate_confirm_token(self.secret_key, &request.confirm_token) {
            Some(id) => {
                User::confirm(self.conn, id)?;
                Ok(ConfirmNewUserResponse)
            }
            None => {
                Err(ServiceError::InvalidConfirmToken.into())
            }
        }
    }

    // Get the user for a request token
    fn current_user(&self, request: &CurrentUserRequest) -> Result<CurrentUserResponse, Error> {
        if let Some(id) = validate_access_token(self.secret_key, &request.access_token) {
            if let Some(user) = User::find(self.conn, id)? {
                return Ok(CurrentUserResponse{
                    identifier: user.id,
                    name: user.name,
                    email: user.email,
                })
            }
        }
        Err(ServiceError::PermissionDenied.into())
    }
}

// TODO: Figure out how to merge the validate copy pasta
fn validate_confirm_token(key: &[u8], token: &str) -> Option<Uuid> {
    let token_result = jwt::decode::<ConfirmTokenClaim>(token, key, &jwt::Validation::default());
    println!("{:?}", token_result);
    token_result.ok()
        .and_then(|token| 
            if token.claims.confirm_token {
                Uuid::parse_str(&token.claims.sub).ok()
            } else {
                None
            }
        )
}

fn validate_access_token(key: &[u8], token: &str) -> Option<Uuid> {
    if let Some(data) = jwt::decode::<AccessTokenClaim>(token, key, &jwt::Validation::default()).ok() {
        if data.claims.access_token {
            return Uuid::parse_str(&data.claims.sub).ok()
        }
    }
    None
}

fn validate_refresh_token(key: &[u8], token: &str) -> Option<Uuid> {
    if let Some(data) = jwt::decode::<RefreshTokenClaim>(token, key, &jwt::Validation::default()).ok() {
        if data.claims.refresh_token {
            return Uuid::parse_str(&data.claims.sub).ok()
        }
    }
    None
}

fn encode_token<T: Serialize>(key: &[u8], claims: T) -> String {
    // TODO: handle error correctly
    jwt::encode(&jwt::Header::default(), &claims, key).unwrap_or("".into())
}

fn access_token_response(key: &[u8], user: User) -> AccessTokenResponse {
    AccessTokenResponse{
        access_token: encode_token(key, 
            AccessTokenClaim{sub: user.id.simple().to_string(), access_token: true}
        ),
        refresh_token: encode_token(key,
            RefreshTokenClaim{sub: user.id.simple().to_string(), refresh_token: true}
        ),
        token_type: "bearer".into(),
        expires_in: 0,
    }
}