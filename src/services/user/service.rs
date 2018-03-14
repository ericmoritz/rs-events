
use uuid::Uuid;
use super::*;
use diesel::pg::PgConnection;
use models::user::{NewUser, User};
use jsonwebtoken as jwt;
use std::default::Default;
use failure::Error;
use serde::ser::Serialize;

pub struct Service {
    conn: PgConnection,
    secret_key: String,
}

impl Service {
    pub fn new(conn: PgConnection, secret_key: String) -> Service {
        Service{conn, secret_key}
    }
}
impl Actor for Service {
    type Context = SyncContext<Self>;
}
impl Handler<RegisterRequest> for Service {
    type Result = Result<RegisterResponse, Error>;

    fn handle(&mut self, request: RegisterRequest, _: &mut Self::Context) -> Self::Result {
        self.register(&request)
    }
}
impl Handler<ConfirmNewUserRequest> for Service {
    type Result = Result<ConfirmNewUserResponse, Error>;

    fn handle(&mut self, request: ConfirmNewUserRequest, _: &mut Self::Context) -> Self::Result {
        self.confirm_new_user(&request)
    }
}
impl Handler<PasswordGrantRequest> for Service {
    type Result = Result<AccessTokenResponse, Error>;

    fn handle(&mut self, request: PasswordGrantRequest, _: &mut Self::Context) -> Self::Result {
        self.password_grant(&request)
    }
}
impl Handler<CurrentUserRequest> for Service {
    type Result = Result<CurrentUserResponse, Error>;

    fn handle(&mut self, request: CurrentUserRequest, _: &mut Self::Context) -> Self::Result {
        self.current_user(&request)
    }
}

impl UserService for Service {
   // login is called to get an access token using a un/pw
    fn password_grant(&self, request: &PasswordGrantRequest) -> Result<AccessTokenResponse, Error> {
        let user: User = User::login(&self.conn, &request.name, &request.password)?
            .ok_or(ServiceError::PermissionDenied)?;

        Ok(access_token_response(self.secret_key.as_bytes(), &user))
    }

    // refresh_token_grant is called to get a new access token
    fn refresh_token_grant(&self, request: &RefreshGrantRequest) -> Result<AccessTokenResponse, Error> {

        let id = validate_refresh_token(self.secret_key.as_bytes(), &request.refresh_token)
            .ok_or(ServiceError::PermissionDenied)?;
        
        let user = User::find(&self.conn, id)?
            .ok_or(ServiceError::PermissionDenied)?;
        
        Ok(access_token_response(self.secret_key.as_bytes(), &user))
    }

    // register is called when registering a new user
    fn register(&self, request: &RegisterRequest) -> Result<RegisterResponse, Error> {
        let user = NewUser{
            id: Uuid::new_v4(),
            name: request.name.clone(),
            password: request.password.clone(),
            email: request.email.clone(),
        }
            .create(&self.conn)?
            .ok_or(ServiceError::UserExists)?;

        Ok(RegisterResponse{
                confirm_token: encode_token(self.secret_key.as_bytes(), 
                    ConfirmTokenClaim{sub: user.id.simple().to_string(), confirm_token: true}
                )
        })
    }

    // confirm_new_user
    fn confirm_new_user(&self, request: &ConfirmNewUserRequest) -> Result<ConfirmNewUserResponse, Error> {
        let id = validate_confirm_token(self.secret_key.as_bytes(), &request.confirm_token)
            .ok_or(ServiceError::InvalidConfirmToken)?;

        User::confirm(&self.conn, id)?;

        Ok(ConfirmNewUserResponse)
    }

    // Get the user for a request token
    fn current_user(&self, request: &CurrentUserRequest) -> Result<CurrentUserResponse, Error> {
        let id = validate_access_token(self.secret_key.as_bytes(), &request.access_token)
            .ok_or(ServiceError::PermissionDenied)?;
        let user =  User::find(&self.conn, id)?
            .ok_or(ServiceError::PermissionDenied)?;
        
        Ok(CurrentUserResponse{
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
    if let Ok(data) = jwt::decode::<AccessTokenClaim>(token, key, &jwt::Validation::default()) {
        if data.claims.access_token {
            return Uuid::parse_str(&data.claims.sub).ok()
        }
    }
    None
}

fn validate_refresh_token(key: &[u8], token: &str) -> Option<Uuid> {
    if let Ok(data) = jwt::decode::<RefreshTokenClaim>(token, key, &jwt::Validation::default()) {
        if data.claims.refresh_token {
            return Uuid::parse_str(&data.claims.sub).ok()
        }
    }
    None
}

fn encode_token<T: Serialize>(key: &[u8], claims: T) -> String {
    // TODO: handle error correctly
    jwt::encode(&jwt::Header::default(), &claims, key).unwrap_or_else(|_| "".into())
}

fn access_token_response(key: &[u8], user: &User) -> AccessTokenResponse {
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