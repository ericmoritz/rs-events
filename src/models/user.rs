use diesel::prelude::*;
use diesel;
use uuid::Uuid;
use schema::users;
use libpasta::{hash_password, verify_password};

#[derive(Queryable)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub confirmed: bool,
}

impl User {
    pub fn find(conn: &PgConnection, user_id: Uuid) -> QueryResult<Option<User>> {
        use schema::users::dsl::*;

        users.filter(id.eq(user_id))
            .filter(confirmed.eq(true))
            .get_result(conn)
            .optional()
    }

    pub fn confirm(conn: &PgConnection, user_id: Uuid) -> QueryResult<usize> {
        use schema::users::dsl::*;
        diesel::update(users)
            .filter(id.eq(user_id))
            .set(confirmed.eq(true))
            .execute(conn)
    }

    pub fn login(conn: &PgConnection, username: &String, pass: &String) -> QueryResult<Option<User>> {
        use schema::users::dsl::*;

        let result: Option<User> = users
            .filter(name.eq(username))
            .filter(confirmed.eq(true))
            .get_result(conn)
            .optional()?;
        
        Ok(result.and_then(|x| if verify_password(&x.password, pass.clone()) {
            Some(x)
        } else {
            None
        }))
    }
}

#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String
}

impl NewUser {

    pub fn create(mut self, conn: &PgConnection) -> QueryResult<Option<User>> {
         use schema::users::dsl::*;
         self.password = hash_password(self.password.clone());
 
         conn.transaction(|| {
            let user = users.filter(name.eq(self.name.clone()))
                .get_result::<User>(conn)
                .optional()?;
            
            match user {
                Some(_) => Ok(None),
                None => diesel::insert_into(users)
                    .values(&self)
                    .get_result::<User>(conn)
                    .optional()
            }
         })
    }
}
