extern crate rs_events;
extern crate diesel;
extern crate dotenv;
extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate serde_json;

use rs_events::services;
use rs_events::db;
use services::user::service::Service;
use services::user::*;
use diesel::prelude::*;

use actix::prelude::*;

use actix_web::*;
use actix_web::test::TestServer;


#[test]
fn test() { 
    use rs_events::schema::users::dsl::*;
 
    let conn = db::connection();
    

    // Delete the test-user
    diesel::delete(users)
        .filter(name.eq("test-user"))
        .execute(&conn)
        .expect("Error deleting test user");
    
    //TODO mock out the I/O
    let mut srv = TestServer::with_factory(|| {
        let addr = SyncArbiter::start(3, || {
            Service::new(db::connection(), String::from("test-secret"))
        });

        web_app::new(addr)
    });

    // Register a new user
    let request = srv.client(Method::POST, "/register")
        .json(
            RegisterRequest{
                name: String::from("test-user"),
                password: String::from("test-pass"),
                email: String::from("test@example.com"),
            }
        ).expect("/register request");
   
    let response = srv.execute(request.send()).expect("/register response");

    assert!(response.status().is_success());
    
    let data: RegisterResponse = serde_json::from_slice(
        &*srv.execute(response.body()).unwrap()
    ).unwrap();
    

    // Confirm the new user
    let request = srv.client(Method::POST, "/confirm")
        .json(
            ConfirmNewUserRequest{
                confirm_token: data.confirm_token,
            }
        ).unwrap();

    let response = srv.execute(request.send()).unwrap();

    println!("{:?}", response);
    assert!(response.status().is_success());

    let _data: ConfirmNewUserResponse = serde_json::from_slice(
        &*srv.execute(response.body()).unwrap()
    ).unwrap();

    //TODO login using the password grant
    //TODO get the current user
    //TODO refresh the access token
    //TODO get the current user with the new access token
}
