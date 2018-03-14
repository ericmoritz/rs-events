extern crate rs_events;
extern crate diesel;
extern crate dotenv;
extern crate actix;
extern crate futures;

use rs_events::services;
use rs_events::db;
use services::user::service::Service;
use services::user::*;
use diesel::prelude::*;
use rs_events::models::user;

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
    let conn = db::connection();
    let service = Service::new(user::Model::new(&conn), String::from("test-secret"));
    
    let registration = service.register(&RegisterRequest{
            name: String::from("test-user"),
            password: String::from("test-pass"),
            email: String::from("test@example.com"),
    }).expect("Could not register");
    
    let _confirm = service.confirm_new_user(&ConfirmNewUserRequest{
        confirm_token: registration.confirm_token,
    }).expect("Could not confirm");

    let _login = service.password_grant(&PasswordGrantRequest{
        client_id: "test".into(),
        name: "test-user".into(),
        password: "test-pass".into(),
    }).expect("Could Not Login");
}
