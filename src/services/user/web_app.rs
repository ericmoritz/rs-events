use actix::prelude::*;
use actix_web::*;
use actix_web;
use futures::future::Future;
use futures::future;
use services::user;
use services::user::{service, ServiceError};

//TODO add a user/machine readable response for the bad requests
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            &ServiceError::InvalidConfirmToken => httpcodes::HTTPBadRequest.build().body("InvalidConfirmToken").unwrap(),
            &ServiceError::PermissionDenied => httpcodes::HTTPUnauthorized.into(),
            &ServiceError::UserExists => httpcodes::HTTPBadRequest.build().body("UserExists").unwrap(),
        }
    }
}


pub struct State {
    addr: Addr<Syn, service::Service>, 
}

pub fn new(addr: Addr<Syn, service::Service>) -> Application<State> {
    Application::with_state(State{addr: addr})
        .resource("/confirm", |r| r.method(Method::POST).a(confirm))
        .resource("/register", |r| r.method(Method::POST).a(register))
}

pub fn register(req: HttpRequest<State>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let addr = req.state().addr.clone();
    req.json()
        .from_err()
        .and_then(move |rreq: user::RegisterRequest| addr.send(rreq).from_err() )
        .and_then(|resp| httpcodes::HTTPOk.build().json(resp?)).from_err()
        .responder()
}

pub fn confirm(req: HttpRequest<State>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    let addr = req.state().addr.clone();
    req.json()
        .from_err()
        .and_then(move |rreq: user::ConfirmNewUserRequest| addr.send(rreq).from_err() )
        .and_then(|resp| httpcodes::HTTPOk.build().json(resp?))
        .from_err()
        .responder()
}

