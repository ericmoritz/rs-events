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

// This is the basic I/O trait so we can mock it later
pub trait UserModel {
    fn find(&self, user_id: Uuid) -> QueryResult<Option<User>>;
    fn confirm(&self, user_id: Uuid) -> QueryResult<usize>;
    fn login(&self, username: &str, pass: &str) -> QueryResult<Option<User>>;
    fn create(&self, user: NewUser) -> QueryResult<Option<User>>;
}

// This is the model implementation
pub struct Model<'a> {
    conn: &'a PgConnection
}
impl<'a> Model<'a> {
    pub fn new(conn: &'a PgConnection) -> Self {
        Model{conn}
    }
}
impl<'a> UserModel for Model<'a> {
    fn find(&self, user_id: Uuid) -> QueryResult<Option<User>> {
        use schema::users::dsl::*;

        users.filter(id.eq(user_id))
            .filter(confirmed.eq(true))
            .get_result(self.conn)
            .optional()
    }

    fn confirm(&self, user_id: Uuid) -> QueryResult<usize> {
        use schema::users::dsl::*;
        diesel::update(users)
            .filter(id.eq(user_id))
            .set(confirmed.eq(true))
            .execute(self.conn)
    }

    fn login(&self, username: &str, pass: &str) -> QueryResult<Option<User>> {
        use schema::users::dsl::*;

        let result: Option<User> = users
            .filter(name.eq(username))
            .filter(confirmed.eq(true))
            .get_result(self.conn)
            .optional()?;
        
        Ok(result.and_then(|x| if verify_password(&x.password, pass.into()) {
            Some(x)
        } else {
            None
        }))
    }

    fn create(&self, mut new_user: NewUser) -> QueryResult<Option<User>> {
         use schema::users::dsl::*;
         new_user.password = hash_password(new_user.password.clone());
 
         self.conn.transaction(|| {
            let user = users.filter(name.eq(new_user.name.clone()))
                .get_result::<User>(self.conn)
                .optional()?;
            
            match user {
                Some(_) => Ok(None),
                None => diesel::insert_into(users)
                    .values(&new_user)
                    .get_result::<User>(self.conn)
                    .optional()
            }
         })
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
