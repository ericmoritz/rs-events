//! implements an `IOModel` for Postgres
use super::{IOModel, NewUser, User};
use diesel;
use diesel::prelude::*;
use libpasta::{hash_password, verify_password};
use uuid::Uuid;

pub struct PgModel {
    // TODO: Make this generic
    conn: PgConnection,
}
impl PgModel {
    pub fn new(conn: PgConnection) -> Self {
        PgModel { conn }
    }
}
impl IOModel for PgModel {
    fn find(&self, user_id: &Uuid) -> QueryResult<Option<User>> {
        use schema::users::dsl::*;

        users
            .filter(id.eq(user_id))
            .filter(confirmed.eq(true))
            .get_result(&self.conn)
            .optional()
    }

    fn confirm(&self, user_id: &Uuid) -> QueryResult<usize> {
        use schema::users::dsl::*;
        diesel::update(users)
            .filter(id.eq(user_id))
            .set(confirmed.eq(true))
            .execute(&self.conn)
    }

    fn verify_login(&self, username: &str, pass: &str) -> QueryResult<Option<User>> {
        use schema::users::dsl::*;

        let result: Option<User> = users
            .filter(name.eq(username))
            .filter(confirmed.eq(true))
            .get_result(&self.conn)
            .optional()?;

        // TODO: move verify_password to the trait
        Ok(result.and_then(|x| {
            if verify_password(&x.password, pass.into()) {
                Some(x)
            } else {
                None
            }
        }))
    }

    fn create(&self, new_user: &NewUser) -> QueryResult<Option<User>> {
        use schema::users::dsl::*;

        // TODO: move this to the trait
        let hash = hash_password(String::from(new_user.password));
        let new_user = &NewUser {
            password: &hash,
            ..*new_user
        };

        self.conn.transaction(|| {
            let user = users
                .filter(name.eq(new_user.name))
                .get_result::<User>(&self.conn)
                .optional()?;

            match user {
                Some(_) => Ok(None),
                None => diesel::insert_into(users)
                    .values(new_user)
                    .get_result::<User>(&self.conn)
                    .optional(),
            }
        })
    }
}
