use uuid::Uuid;
use super::*;
use models::user::{NewUser, User};
use models::user::IOModel;
use models::user::pg::PgModel;
use jsonwebtoken as jwt;
use std::default::Default;
use serde::ser::Serialize;

pub struct Service<'a> {
    // TODO: make this generic so we can mock it out
    model: &'a PgModel<'a>,
    secret_key: &'a [u8],
}

impl<'a> Service<'a> {
    pub fn new(model: &'a PgModel<'a>, secret_key: &'a [u8]) -> Service<'a> {
        Service { model, secret_key }
    }

    // login is called to get an access token using a un/pw
    pub fn password_grant(
        &self,
        request: &PasswordGrantRequest,
    ) -> Result<AccessTokenResponse, ServiceError> {
        let user: User = self.model
            .verify_login(request.name, request.password)?
            .ok_or(ServiceError::PermissionDenied)?;

        Ok(access_token_response(self.secret_key, &user))
    }

    // refresh_token_grant is called to get a new access token
    pub fn refresh_token_grant(
        &self,
        request: &RefreshGrantRequest,
    ) -> Result<AccessTokenResponse, ServiceError> {
        let id = validate_refresh_token(self.secret_key, request.refresh_token)
            .ok_or(ServiceError::PermissionDenied)?;

        let user = self.model.find(id)?.ok_or(ServiceError::PermissionDenied)?;

        Ok(access_token_response(self.secret_key, &user))
    }

    // register is called when registering a new user
    pub fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse, ServiceError> {
        let new_user = NewUser {
            id: Uuid::new_v4(),
            name: request.name,
            password: request.password,
            email: request.email,
        };
        let user = self.model
            .create(new_user)?
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

    // confirm_new_user
    pub fn confirm_new_user(
        &self,
        request: &ConfirmNewUserRequest,
    ) -> Result<ConfirmNewUserResponse, ServiceError> {
        let id = validate_confirm_token(self.secret_key, request.confirm_token)
            .ok_or(ServiceError::InvalidConfirmToken)?;

        self.model.confirm(id)?;

        Ok(ConfirmNewUserResponse)
    }

    // Get the user for a request token
    pub fn current_user(
        &self,
        request: &CurrentUserRequest,
    ) -> Result<CurrentUserResponse, ServiceError> {
        let id = validate_access_token(self.secret_key, request.access_token)
            .ok_or(ServiceError::PermissionDenied)?;
        let user = self.model.find(id)?.ok_or(ServiceError::PermissionDenied)?;

        Ok(CurrentUserResponse {
            identifier: user.id,
            name: user.name,
            email: user.email,
        })
    }
}

/////
/// Internal
/////

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
        expires_in: 0,
    }
}
