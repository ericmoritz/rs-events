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
use actix::prelude::*;
use futures::Future;

//#[test]
fn test() {
    use rs_events::schema::users::dsl::*;

    let sys = System::new("test");
    let conn = db::connection();
   
    // Delete the test-user
    diesel::delete(users)
        .filter(name.eq("test-user"))
        .execute(&conn)
        .expect("Error deleting test user");

    //TODO mock out the I/O
    let addr = SyncArbiter::start(3, || {
        Service::new(db::connection(), String::from("test-secret"))
    });
    let addr2 = addr.clone();
    let addr3 = addr.clone();

    let fut = 
        // Try to register the test-user
        addr.send(RegisterRequest{
            name: String::from("test-user"),
            password: String::from("test-pass"),
            email: String::from("test@example.com"),
        })
        // Attempt to confirm the registration
        .and_then(move |registration| {
            let registration = registration.expect("Could not register");
            println!("\n{:?}\n", registration);

            addr.send(ConfirmNewUserRequest{
                confirm_token: registration.confirm_token,
            })
        })
        // Attempt to login as the user
        .and_then(move |confirm| {
            let _ = confirm.expect("Could not confirm the registration");
            addr2.send(PasswordGrantRequest{
                client_id: "test".into(),
                name: "test-user".into(),
                password: "test-pass".into(),
            })
        })
        // Get the current uuid
        .and_then(move |login| {
            let login = login.expect("Could not login");
            println!("{:?}\n", login);
            addr3.send(CurrentUserRequest{
                access_token: login.access_token
            })
        })
        .map(|user| {
            let user = user.expect("Could not get current user");
            assert_eq!(user.name, "test-user");
        })
        .map(|_| {
            Arbiter::system().do_send(actix::msgs::SystemExit(0));
        }).map_err(|_| ());
        
    Arbiter::handle().spawn(fut);
    sys.run();
}
