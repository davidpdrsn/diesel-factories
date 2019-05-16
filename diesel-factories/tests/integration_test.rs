#[macro_use]
extern crate diesel;
extern crate diesel_factories;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_factories::Factory;

// Tell Diesel what our schema is
mod schema {
    table! {
        users (id) {
            id -> Integer,
            name -> Text,
            age -> Integer,
        }
    }
}

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

#[derive(Factory)]
#[factory_model(User)]
#[table_name = "users"]
pub struct UserFactory<'a> {
    name: String,
    age: i32,
    connection: &'a PgConnection,
}

impl<'a> UserFactory<'a> {
    // Set default values here
    fn new(connection_in: &'a PgConnection) -> UserFactory<'a> {
        UserFactory {
            name: "Bob".into(),
            age: 30,
            connection: connection_in,
        }
    }
}

#[test]
fn basic_test() {
    use crate::schema::users::dsl::*;

    // Connect to the database
    let database_url = "postgres://localhost/diesel_factories_test";
    let con = diesel::pg::PgConnection::establish(&database_url).unwrap();
    con.begin_test_transaction().unwrap();

    // Create a new user using our factory, overriding the default name
    let user = UserFactory::new(&con).name("Alice").insert();
    assert_eq!("Alice", user.name);
    assert_eq!(30, user.age);

    // Verifing that the user is in fact in the database
    let user_from_db = users.filter(id.eq(user.id)).first::<User>(&con).unwrap();
    assert_eq!("Alice", user_from_db.name);
    assert_eq!(30, user_from_db.age);
}
