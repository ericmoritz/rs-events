//! Diesel Schema
table! {
    /// The users table
    users (id) {
        id -> Uuid,
        name -> Varchar,
        email -> Varchar,
        password -> Varchar,
        confirmed -> Bool,
    }
}
