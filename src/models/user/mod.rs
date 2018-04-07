use diesel::prelude::*;
use uuid::Uuid;
use schema::users;

//# Modules

pub mod pg;

//# Structs

/// NewUser is the struct that is used for storing a new user
#[derive(Insertable)]
#[table_name = "users"]
pub struct NewUser<'a> {
    pub id: Uuid,
    pub name: &'a str,
    pub email: &'a str,
    pub password: &'a str,
}

/// User is the struct that repesents a User record
#[derive(Queryable)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub confirmed: bool,
}

//# Traits

/// This trait is the IO interface
pub trait IOModel {
    /// Find a confirmed user
    fn find(&self, user_id: Uuid) -> QueryResult<Option<User>>;

    /// Confirm a user
    fn confirm(&self, user_id: Uuid) -> QueryResult<usize>;

    /// Verify a login
    fn verify_login(&self, username: &str, pass: &str) -> QueryResult<Option<User>>;

    /// Create a new unconfirmed user
    fn create(&self, user: NewUser) -> QueryResult<Option<User>>;
}
