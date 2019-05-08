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

    fn name(mut self, new_value: &str) -> UserFactory<'a> {
        self.name = new_value.to_string();
        self
    }

    fn insert(self) -> User {
        use self::users::dsl::*;
        let res = diesel::insert_into(users)
            .values(((name.eq(&self.name)), age.eq(&self.age)))
            .get_result::<User>(self.connection);

        match res {
            Ok(x) => x,
            Err(err) => panic!("{}", err),
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
    let user = UserFactory::new(&con).name("Alice").insert();
    assert_eq!("Alice", user.name);
    assert_eq!(30, user.age);

    // Verifing that the user is in fact in the database
    let user_from_db = users.filter(id.eq(user.id)).first::<User>(&con).unwrap();
    assert_eq!("Alice", user_from_db.name);
    assert_eq!(30, user_from_db.age);
}
