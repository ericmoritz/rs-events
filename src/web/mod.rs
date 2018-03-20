
use rouille::Response;
use rouille;
use std::io;
use services::user::service::Service as UserService;
use services::user::{UserService as UserServiceTrait, RegisterRequest, RegisterResponse, ServiceError};
use models::user::{Model as UserModel, UserModel as UserModelTrait};
use db;
use uuid::Uuid;

#[derive(Serialize, Debug)]
struct Status<'a> {
    pub status: &'a str,
}

impl From<RegisterResponse> for Response {
    fn from(result: RegisterResponse) -> Self {
        Response::json(&result)
    }
}
impl From<ServiceError> for Response {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::InvalidConfirmToken => Response::text("InvalidConfirmToken").with_status_code(400),
            ServiceError::PermissionDenied => Response::text("").with_status_code(403),
            ServiceError::UserExists => Response::text("UserExists").with_status_code(403),
            ServiceError::Other => Response::text("").with_status_code(500),
        }
    }
}


pub fn run() {

    eprintln!("Listening on 0.0.0.0:8080");
    rouille::start_server("0.0.0.0:8080", |request| {
        rouille::log(&request, io::stderr(), || {
            router!(request,

                (GET) (/status) => {
                    // Attempt to query a user to check if the db is up
                    let conn = db::connection();
                    let model = UserModel::new(&conn);
                    let status = model.find(Uuid::new_v4())
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
                    let conn = db::connection();
                    let user_service = UserService::new(UserModel::new(&conn), b"....");
                    let data: Json = try_or_400!(rouille::input::json_input(request));
                    
                    let req = RegisterRequest{
                        name: &data.name,
                        password: &data.password,
                        email: &data.email,
                    };
                    user_service.register(&req)
                        .map(Response::from)
                        .unwrap_or_else(Response::from)
                },

                (GET) (/oauth/register/confirm) => {
                    rouille::Response::text("").with_status_code(501)
                },
                (POST) (/oauth/access_token) => {
                    rouille::Response::text("").with_status_code(501)
                },

                (GET) (/oauth/me) => {
                    rouille::Response::text("").with_status_code(501)
                },
                _ => Response::empty_404()
            )
        })
    })
}
