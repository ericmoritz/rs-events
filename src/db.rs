//! DB utils
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use diesel::prelude::*;

/// uses the `DATABASE_URL` env var to connect to a postgres DB
pub fn connection() -> PgConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}
