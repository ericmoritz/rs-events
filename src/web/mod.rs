//! This is the initial MVP of the events service to get the BDD tests to work
use rouille::Response;
use rouille;
use rouille::input::post;
use std::io;
use services::user;
use services::user::UserService;
use models::user::{Model as UserModel, UserModel as UserModelTrait};
use db;
use uuid::Uuid;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::collections::HashMap;
use std::iter::FromIterator;


#[derive(Serialize, Debug)]
struct Status<'a> {
    pub status: &'a str,
}

///
/// Runs a web server that passes the BDD tests
/// 
pub fn run() {

    eprintln!("Listening on 0.0.0.0:8080");
    rouille::start_server("0.0.0.0:8080", |request| {

        rouille::log(&request, io::stderr(), || {
            let conn = db::connection();
            let user_model = UserModel::new(&conn);
            let user_service = user::service::Service::new(UserModel::new(&conn), b"....");

            router!(request,

                (GET) (/status) => {
                    let status = user_model.find(Uuid::new_v4())
                        .map(|_| 
                            Status{status: "up"})
                        .unwrap_or_else(|_| 
                            Status{status: "down"});
                        
                    rouille::Response::json(&status)
                },

                (POST) (/oauth/register) => {
                    #[derive(Deserialize)]
                    struct Json {
                        name: String,
                        password: String,
                        email: String,
                    }
                    let data: Json = try_or_400!(rouille::input::json_input(request));
                    
                    let req = user::RegisterRequest{
                        name: &data.name,
                        password: &data.password,
                        email: &data.email,
                    };
                    user_service.register(&req)
                        .map(Response::from)
                        .unwrap_or_else(Response::from)
                },

                (GET) (/oauth/register/confirm) => {
                    let confirm_token: String = try_or_400!(
                        request.get_param("confirm_token")
                            .ok_or(RunError::MissingConfirmToken)
                    );
                    let req = user::ConfirmNewUserRequest{
                        confirm_token: confirm_token
                    };
                    user_service.confirm_new_user(&req)
                        .map(Response::from)
                        .unwrap_or_else(Response::from)
                },

                (POST) (/oauth/token) => {
                    let form = try_or_400!(post::raw_urlencoded_post_input(request));
                    let grant_type = try_or_400!(find_grant_type(&form));
                    match grant_type {
                        GrantType::Password => {
                            let req = try_or_400!(form_to_password_grant(&form));
                            user_service.password_grant(&req)
                                .map(Response::from)
                                .unwrap_or_else(Response::from)
                        },
                        GrantType::Refresh => {
                            let req = try_or_400!(form_to_refresh_grant(&form));
                            
                            user_service.refresh_token_grant(&req)
                                .map(Response::from)
                                .unwrap_or_else(Response::from)
                        }
                    }
                    
                },

                (GET) (/oauth/me) => {
                    let access_token = request.header("Authorization")
                        .and_then(move |x| x.get(7..)) // Get everything after "Bearer "
                        .unwrap_or("");

                    let req = user::CurrentUserRequest{
                        access_token: String::from(access_token)
                    };
                    user_service.current_user(&req)
                        .map(Response::from)
                        .unwrap_or_else(Response::from)
                },
                
                _ => Response::empty_404()
            )
        })
    })
}

///
/// This is a private Error type for things that can go wrong in run()
/// 
#[derive(Debug, PartialEq)]
enum RunError {
    MissingConfirmToken,
    MissingPassword,
    MissingUsername,
    MissingRefreshToken,
    InvalidGrantType,
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for RunError {
    fn description(&self) -> &str {
        use self::RunError::*;

        match *self {
            MissingUsername => "missing username",
            MissingPassword => "missing password",
            MissingRefreshToken => "missing refresh_token",
            MissingConfirmToken => "missing confirm token",
            InvalidGrantType => "invalid grant type",
            
        }
    }
}

///
/// # Converters
/// 
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

impl From<user::ServiceError> for Response {
    fn from(err: user::ServiceError) -> Self {
        match err {
            user::ServiceError::InvalidConfirmToken => Response::text("InvalidConfirmToken").with_status_code(400),
            user::ServiceError::PermissionDenied => Response::text("").with_status_code(403),
            user::ServiceError::UserExists => Response::text("UserExists").with_status_code(403),
            user::ServiceError::Other => Response::text("").with_status_code(500),
        }
    }
}


///
/// This is a enum to represent the grant_type strings, "password" and "refresh_token"
///
/// Note: We may want to move this to the service module
#[derive(Debug, PartialEq)]
enum GrantType {
    Password,
    Refresh
}

impl FromStr for GrantType {
    type Err = RunError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "password" => Ok(GrantType::Password),
            "refresh_token" => Ok(GrantType::Refresh),
            _ => Err(RunError::InvalidGrantType)
        }
    }
}
#[test]
fn test_grant_type_from_str() {
    assert_eq!(
            GrantType::from_str("password").unwrap(), GrantType::Password
    );
}

///
/// # Helpers
/// 

///
/// Finds the grant_type in the Vector of form fields
///
fn find_grant_type(fields: &Vec<(String, String)>) -> Result<GrantType, RunError> {
    for &(ref k, ref v) in fields.iter() {
        if k == "grant_type" {
            return GrantType::from_str(&v);
        }
    }
    Err(RunError::InvalidGrantType)
}
#[test]
fn test_find_grant_type() {
    assert_eq!(
        find_grant_type(&vec![("x".into(), "y".into()), ("grant_type".into(), "password".into()), ("a".into(), "b".into())]).unwrap(),
        GrantType::Password
    );

    assert_eq!(
        find_grant_type(&vec![("x".into(), "y".into()), ("grant_type".into(), "refresh_token".into()), ("a".into(), "b".into())]).unwrap(),
        GrantType::Refresh
    );


    assert_eq!(
        find_grant_type(&vec![("x".into(), "y".into()), ("a".into(), "b".into())]).unwrap_err(),
        RunError::InvalidGrantType
    );
}

///
/// Converts the Form Fields to a PasswordGrantRequest  
///
fn form_to_password_grant<'a>(fields: &'a Vec<(String, String)>) -> Result<user::PasswordGrantRequest<'a>, RunError> {
    let fields: HashMap<&str, &str> = HashMap::from_iter(
        fields.iter().map(|&(ref k, ref v)| {
            let k: &str = k;
            let v: &str = v;
            (k,v)
        }));
    let name = fields.get("username").ok_or(RunError::MissingUsername)?;
    let password = fields.get("password").ok_or(RunError::MissingPassword)?;

    Ok(user::PasswordGrantRequest{
        name: name,
        password: password,
    })
}
#[test]
fn test_form_to_password_grant() {
    assert_eq!(
        form_to_password_grant(&vec![
            ("grant_type".into(), "password".into()),
            ("username".into(), "test-user".into()),
            ("password".into(), "test-password".into()),
        ]).unwrap(),
        user::PasswordGrantRequest{
            name: "test-user".into(),
            password: "test-password".into(),
        }
    );

    assert_eq!(
        form_to_password_grant(&vec![]).unwrap_err(),
        RunError::MissingUsername
    );

    assert_eq!(
        form_to_password_grant(&vec![("username".into(), "test-user".into())]).unwrap_err(),
        RunError::MissingPassword
    );

    assert_eq!(
        form_to_password_grant(&vec![("password".into(), "test-pass".into())]).unwrap_err(),
        RunError::MissingUsername
    );
}

///
/// Converts the Form Fields into a RefreshGrantRequest
///
fn form_to_refresh_grant<'a>(fields: &'a Vec<(String, String)>) -> Result<user::RefreshGrantRequest<'a>, RunError> {
    let fields: HashMap<&str, &str> = HashMap::from_iter(
        fields.iter().map(|&(ref k, ref v)| {
            let k: &str = k;
            let v: &str = v;
            (k,v)
        }));
        
    let token = fields.get("refresh_token").ok_or(RunError::MissingRefreshToken)?;
    
    Ok(user::RefreshGrantRequest{
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
        user::RefreshGrantRequest{
            refresh_token: "12345".into()
        }
    );

    assert_eq!(
        form_to_refresh_grant(&vec![]).unwrap_err(),
        RunError::MissingRefreshToken
    );
}
