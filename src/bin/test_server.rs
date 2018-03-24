extern crate rs_events;
use rs_events::web;
use std::process::Command;

fn main() {
    // Run the migrations
    Command::new("diesel")
        .arg("setup")
        .output()
        .expect("Unable to setup database");
    Command::new("diesel")
        .arg("migrations")
        .arg("run")
        .output()
        .expect("Unable to run migrations");
        
    // Then start the web server
    web::run();
}
