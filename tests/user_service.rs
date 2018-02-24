extern crate rs_events;
extern crate diesel;
extern crate dotenv;

use rs_events::services;
use services::user::service::Service;
use services::user::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;

#[test]
fn test() {
    use rs_events::schema::users::dsl::*;

    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let conn = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));
    
    // Delete the test-user
    diesel::delete(users)
        .filter(name.eq("test-user"))
        .execute(&conn)
        .expect("Error deleting test user");


    let service = Service::new(&conn, b"test-secret");

    // Try to register the test-user
    let registration = service.register(&RegisterRequest{
        name: String::from("test-user"),
        password: String::from("test-pass"),
        email: String::from("test@example.com"),
    }).expect("Registration failed");

    println!("\n{:?}\n", registration);
    
    // Confirm the user
    let confirm = service.confirm_new_user(&ConfirmNewUserRequest{
        confirm_token: registration.confirm_token,
    }).expect("Unable to confirm the user");

     println!("{:?}\n", confirm);

    // Attempt to login the user
    let login = service.password_grant(&PasswordGrantRequest{
        client_id: "test".into(),
        name: "test-user".into(),
        password: "test-pass".into(),
    }).expect("Could not login");

    println!("{:?}\n", login);

    // Try to get the current user's data
    let user = service.current_user(&CurrentUserRequest{
        access_token: login.access_token,
    }).expect("Could not get current user");

    println!("{:?}\n", user);
    assert_eq!(user.name, "test-user");

    // Try to refresh the token
    let refresh = service.refresh_token_grant(&RefreshGrantRequest{
        refresh_token: login.refresh_token,
    }).expect("Could not refresh token");

    // Try to get the current user's data
    let user = service.current_user(&CurrentUserRequest{
        access_token: refresh.access_token,
    }).expect("Could not get current user");

    assert_eq!(user.name, "test-user")

   

}