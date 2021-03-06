//! This is the initial MVP of the events service to get the BDD tests to work
use db;
use models::user::IOModel;
use models::user::pg::PgModel as UserModel;
use rouille;
use rouille::input::post;
use rouille::{Request, Response};
use services::user;
use services::user::Service as UserService;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io;
use std::iter::FromIterator;
use std::str::FromStr;
use uuid::Uuid;

//
// Runs a web server that passes the BDD tests
//
pub fn run() {
    eprintln!("Listening on 0.0.0.0:8080");
    rouille::start_server("0.0.0.0:8080", |request| {
        rouille::log(request, io::stderr(), || {
            let conn = &db::connection();
            let user_model = &UserModel::new(conn);
            let user_service = &UserService::new(user_model, b"....");

            router!(request,

                (GET)  (/status) => { status(user_model) },
                (POST) (/oauth/register) => { oauth_register(user_service, request) },
                (GET)  (/oauth/register/confirm) => { oauth_register_confirm(user_service, request) },
                (POST) (/oauth/token) => { oauth_token(user_service, request) },
                (GET)  (/oauth/me) => { me(user_service, request) },
                _ => Response::empty_404()
            )
        })
    })
}
//
// Handlers
//
#[derive(Serialize, Debug)]
struct Status<'a> {
    pub status: &'a str,
}

/// this is the status endpoint
fn status(user_model: &UserModel) -> Response {
    let status = user_model
        .find(&Uuid::new_v4())
        .map(|_| Status { status: "up" })
        .unwrap_or_else(|_| Status { status: "down" });

    Response::json(&status)
}

#[derive(Deserialize)]
struct RegisterForm {
    name: String,
    password: String,
    email: String,
}

/// this is the user registration endpoint
///
/// This accepts a json POST of [`RegisterForm`]
fn oauth_register(user_service: &UserService, request: &Request) -> Response {
    let data: RegisterForm = try_or_400!(rouille::input::json_input(request));

    let req = user::RegisterRequest {
        name: &data.name,
        password: &data.password,
        email: &data.email,
    };
    user_service
        .register(&req)
        .map(Response::from)
        .unwrap_or_else(Response::from)
}

/// this is the user confirmation endpoint
///
/// This is a GET request for a query string of `?confirm_token`
fn oauth_register_confirm(user_service: &UserService, request: &Request) -> Response {
    let confirm_token: String = try_or_400!(
        request
            .get_param("confirm_token")
            .ok_or(WebError::MissingConfirmToken)
    );
    let req = &user::ConfirmNewUserRequest {
        confirm_token: &confirm_token,
    };
    user_service
        .confirm_new_user(req)
        .map(Response::from)
        .unwrap_or_else(Response::from)
}

/// this is the oauth token endpoint for making password or refresh grants against
///
/// This follows the protocol set up by the following specs
///
///  - [password grant](https://tools.ietf.org/html/rfc6749#section-4.3.2)
///  - [refresh grant](https://tools.ietf.org/html/rfc6749#section-6)
///
fn oauth_token(user_service: &UserService, request: &Request) -> Response {
    let form = &try_or_400!(post::raw_urlencoded_post_input(request));
    let grant_type = try_or_400!(find_grant_type(form));
    match grant_type {
        GrantType::Password => {
            let req = &try_or_400!(form_to_password_grant(form));
            user_service
                .password_grant(req)
                .map(Response::from)
                .unwrap_or_else(Response::from)
        }
        GrantType::Refresh => {
            let req = &try_or_400!(form_to_refresh_grant(form));

            user_service
                .refresh_token_grant(req)
                .map(Response::from)
                .unwrap_or_else(Response::from)
        }
    }
}

/// The current user handler
///
/// This requires a `Authorization: Bearer {access_token}` header to make the request
fn me(user_service: &UserService, request: &Request) -> Response {
    let access_token = request.header("Authorization")
        .and_then(move |x| x.get(7..)) // Get everything after "Bearer "
        .unwrap_or("");

    let req = &user::CurrentUserRequest { access_token };
    user_service
        .current_user(req)
        .map(Response::from)
        .unwrap_or_else(Response::from)
}

// Cenverters
//
impl From<user::CurrentUserResponse> for Response {
    fn from(result: user::CurrentUserResponse) -> Self {
        Response::json(&result)
    }
}

impl From<user::AccessTokenResponse> for Response {
    fn from(result: user::AccessTokenResponse) -> Self {
        Response::json(&result)
    }
}

impl From<user::ConfirmNewUserResponse> for Response {
    fn from(result: user::ConfirmNewUserResponse) -> Self {
        Response::json(&result)
    }
}

impl From<user::RegisterResponse> for Response {
    fn from(result: user::RegisterResponse) -> Self {
        Response::json(&result)
    }
}

///
/// This is a private Error type for things that can go wrong
///
#[derive(Debug, PartialEq)]
enum WebError {
    MissingConfirmToken,
    MissingPassword,
    MissingUsername,
    MissingRefreshToken,
    InvalidGrantType,
}

impl fmt::Display for WebError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for WebError {
    fn description(&self) -> &str {
        use self::WebError::*;

        match *self {
            MissingUsername => "missing username",
            MissingPassword => "missing password",
            MissingRefreshToken => "missing refresh_token",
            MissingConfirmToken => "missing confirm token",
            InvalidGrantType => "invalid grant type",
        }
    }
}

impl From<user::ServiceError> for Response {
    fn from(err: user::ServiceError) -> Self {
        use services::user::ServiceError::*;
        match err {
            InvalidConfirmToken => Response::text("InvalidConfirmToken").with_status_code(400),
            PermissionDenied => Response::text("").with_status_code(403),
            UserExists => Response::text("UserExists").with_status_code(403),
            DBError(_) => Response::text("").with_status_code(500),
        }
    }
}

///
/// This is a enum to represent the `grant_type` strings, `"password"` and `"refresh_token"`
///
/// Note: We may want to move this to the service module
#[derive(Debug, PartialEq)]
enum GrantType {
    Password,
    Refresh,
}

impl FromStr for GrantType {
    type Err = WebError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "password" => Ok(GrantType::Password),
            "refresh_token" => Ok(GrantType::Refresh),
            _ => Err(WebError::InvalidGrantType),
        }
    }
}
#[test]
fn test_grant_type_from_str() {
    assert_eq!(
        GrantType::from_str("password").unwrap(),
        GrantType::Password
    )
}

///
/// # Helpers
///

///
/// Finds the `grant_type` in the Vector of form fields
///
type Fields = [(String, String)];
fn find_grant_type(fields: &Fields) -> Result<GrantType, WebError> {
    for &(ref k, ref v) in fields.iter() {
        if k == "grant_type" {
            return GrantType::from_str(v);
        }
    }
    Err(WebError::InvalidGrantType)
}
#[test]
fn test_find_grant_type() {
    assert_eq!(
        find_grant_type(&vec![
            ("x".into(), "y".into()),
            ("grant_type".into(), "password".into()),
            ("a".into(), "b".into()),
        ]).unwrap(),
        GrantType::Password
    );

    assert_eq!(
        find_grant_type(&vec![
            ("x".into(), "y".into()),
            ("grant_type".into(), "refresh_token".into()),
            ("a".into(), "b".into()),
        ]).unwrap(),
        GrantType::Refresh
    );

    assert_eq!(
        find_grant_type(&vec![("x".into(), "y".into()), ("a".into(), "b".into())]).unwrap_err(),
        WebError::InvalidGrantType
    );
}

fn form_to_map(fields: &Fields) -> HashMap<&str, &str> {
    HashMap::from_iter(fields.iter().map(|&(ref k, ref v)| {
        let k: &str = k;
        let v: &str = v;
        (k, v)
    }))
}

///
/// Converts the Form Fields to a `PasswordGrantRequest`
///
fn form_to_password_grant(
    fields: &[(String, String)],
) -> Result<user::PasswordGrantRequest, WebError> {
    let fields = form_to_map(fields);
    let username = fields.get("username").ok_or(WebError::MissingUsername)?;
    let password = fields.get("password").ok_or(WebError::MissingPassword)?;

    Ok(user::PasswordGrantRequest { username, password })
}
#[test]
fn test_form_to_password_grant() {
    assert_eq!(
        form_to_password_grant(&vec![
            ("grant_type".into(), "password".into()),
            ("username".into(), "test-user".into()),
            ("password".into(), "test-password".into()),
        ]).unwrap(),
        user::PasswordGrantRequest {
            username: "test-user".into(),
            password: "test-password".into(),
        }
    );

    assert_eq!(
        form_to_password_grant(&vec![]).unwrap_err(),
        WebError::MissingUsername
    );

    assert_eq!(
        form_to_password_grant(&vec![("username".into(), "test-user".into())]).unwrap_err(),
        WebError::MissingPassword
    );

    assert_eq!(
        form_to_password_grant(&vec![("password".into(), "test-pass".into())]).unwrap_err(),
        WebError::MissingUsername
    );
}

/// Converts the Form Fields into a `RefreshGrantRequest`
fn form_to_refresh_grant(fields: &Fields) -> Result<user::RefreshGrantRequest, WebError> {
    let fields = form_to_map(fields);
    let token = fields
        .get("refresh_token")
        .ok_or(WebError::MissingRefreshToken)?;

    Ok(user::RefreshGrantRequest {
        refresh_token: token,
    })
}
#[test]
fn test_form_to_refresh_grant() {
    assert_eq!(
        form_to_refresh_grant(&vec![
            ("grant_type".into(), "refesh_token".into()),
            ("refresh_token".into(), "12345".into()),
        ]).unwrap(),
        user::RefreshGrantRequest {
            refresh_token: "12345".into(),
        }
    );

    assert_eq!(
        form_to_refresh_grant(&vec![]).unwrap_err(),
        WebError::MissingRefreshToken
    );
}
