#[macro_use]
extern crate diesel;
extern crate diesel_factories;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_factories::{DefaultFactory, Factory, InsertFactory};

// Tell Diesel what our schema is
table! {
    users (id) {
        id -> Integer,
        name -> Text,
        age -> Integer,
    }
}

// Setup the model. We have to implement `Identifiable`.
#[derive(Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
}

// On a normal Diesel `Insertable` you can derive `Factory`
#[derive(Insertable, Factory)]
#[table_name = "users"]
// And specify which model type the factory is for
#[factory_model(User)]
pub struct UserFactory {
    name: String,
    age: i32,
}

// Set default values. If you don't implement `Default` it wont work.
impl Default for UserFactory {
    fn default() -> UserFactory {
        UserFactory {
            name: "Bob".into(),
            age: 30,
        }
    }
}

#[test]
fn basic_test() {
    use self::users::dsl::*;

    // Connect to the database
    let database_url = "postgres://localhost/diesel_factories_test";
    let con = PgConnection::establish(&database_url).unwrap();
    con.begin_test_transaction();

    // Create a new user using our factory, overriding the default name
    let user = User::default_factory().name("Alice").insert(&con);
    assert_eq!("Alice", user.name);
    assert_eq!(30, user.age);

    // Verifing that the user is in fact in the database
    let user_from_db = users.filter(id.eq(user.id)).first::<User>(&con).unwrap();
    assert_eq!("Alice", user_from_db.name);
    assert_eq!(30, user_from_db.age);
}
