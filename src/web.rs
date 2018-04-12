//! This is the initial MVP of the events service to get the BDD tests to work
use actix::prelude::*;
use actix_web::error;
use actix_web::{http, server, App, AsyncResponder, Form, FutureResponse, HttpMessage, HttpRequest,
                HttpResponse, Json, Query, State};
use db;
use futures::future::Future;
use models::user::pg::PgModel as UserModel;
use services::user;
use services::user::{Service as UserService, ServiceError};

struct AppState {
    user_service: Addr<Syn, UserService>,
}

/// run the web service
pub fn run() {
    let sys = actix::System::new("rs-events");

    // TODO: Use a connection pool

    // Create the user service
    let user_service = SyncArbiter::start(3, || {
        let conn = db::connection();
        let user_model = UserModel::new(conn);
        UserService::new(user_model, "....".into())
    });

    // Define the routes for the service
    server::new(move || {
        App::with_state(AppState {
            user_service: user_service.clone(),
        }).resource("/status", |r| r.method(http::Method::GET).with(status))
            .resource("/oauth/register", |r| {
                r.method(http::Method::POST).with2(oauth_register)
            })
            .resource("/oauth/register/confirm", |r| {
                r.method(http::Method::GET).with2(oauth_register_confirm)
            })
            .resource("/oauth/token", |r| {
                r.method(http::Method::POST).with2(token)
            })
            .resource("/oauth/me", |r| r.method(http::Method::GET).a(me))
    }).bind("0.0.0.0:8080")
        .unwrap()
        .start();

    println!("Started http server: 0.0.0.0:8080");
    let _ = sys.run();
}

/// this is the service status page
fn status(state: State<AppState>) -> FutureResponse<HttpResponse> {
    state
        .user_service
        .send(user::StatusRequest {})
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res?)))
        .responder()
}

/// this is the user registration endpoint
fn oauth_register(
    state: State<AppState>,
    req: Json<user::RegisterRequest>,
) -> FutureResponse<HttpResponse> {
    state
        .user_service
        .send(req.0)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res?)))
        .responder()
}

/// this is the confirmation endpoint for confirming a user
fn oauth_register_confirm(
    state: State<AppState>,
    req: Query<user::ConfirmNewUserRequest>,
) -> FutureResponse<HttpResponse> {
    state
        .user_service
        .send(req.into_inner())
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res?)))
        .responder()
}

/// this is the request that user-agents can post to get an access token using a OAuth 2.0 password or refresh grant
fn token(state: State<AppState>, req: Form<user::TokenRequest>) -> FutureResponse<HttpResponse> {
    state
        .user_service
        .send(req.0)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res?)))
        .responder()
}

/// this is JSON for the user that owns the Access Token in the Auth header
fn me(req: HttpRequest<AppState>) -> FutureResponse<HttpResponse> {
    let state = req.state();

    // TODO: Move this to a middleware or extractor for simplifying this
    let access_token: String = req.headers()
        .get("authorization")
        .and_then(|x| access_token(x.to_str().ok()?))
        .unwrap_or_else(|| "")
        .into();

    // Send the current user request to the backend and convert it to JSON
    state
        .user_service
        .send(user::CurrentUserRequest { access_token })
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res?)))
        .responder()
}

/// Convert the ServiceError's into actix Errors
impl From<ServiceError> for error::Error {
    fn from(x: ServiceError) -> Self {
        match x {
            ServiceError::InvalidConfirmToken => error::ErrorUnauthorized(x),
            ServiceError::PermissionDenied => error::ErrorUnauthorized(x),
            ServiceError::UserExists => error::ErrorConflict(x),
            ServiceError::DBError(e) => error::ErrorInternalServerError(e),
        }
    }
}

/// Extract the access_token from the Auth header if it is valid
fn access_token(x: &str) -> Option<&str> {
    match (x.get(..7), x.get(7..)) {
        (Some("Bearer "), Some(x)) => Some(x),
        _ => None,
    }
}
#[test]
fn test_confirm_token() {
    assert_eq!(confirm_token("Bearer xxx"), Some("xxx"));
}
