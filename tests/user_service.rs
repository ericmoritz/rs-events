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
    let service = Service::new(user::Model::new(&conn), b"test-secret");
    
    let registration = service.register(&RegisterRequest{
            name: "test-user",
            password: "test-pass",
            email: "test@example.com",
    }).expect("Could not register");
    
    let _confirm = service.confirm_new_user(&ConfirmNewUserRequest{
        confirm_token: registration.confirm_token,
    }).expect("Could not confirm");

    let _login = service.password_grant(&PasswordGrantRequest{
        client_id: "test",
        name: "test-user",
        password: "test-pass",
    }).expect("Could Not Login");
}
