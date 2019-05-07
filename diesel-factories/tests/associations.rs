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
        country_id -> Nullable<Integer>,
    }
}

table! {
    countrys (id) {
        id -> Integer,
        name -> Text,
    }
}

// Setup the model. We have to implement `Identifiable`.
#[derive(Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub country_id: Option<i32>,
}

// On a normal Diesel `Insertable` you can derive `Factory`
#[derive(Insertable, Factory)]
#[table_name = "users"]
// And specify which model type the factory is for
#[factory_model(User)]
pub struct UserFactory {
    name: String,
    age: i32,
    country_id: Option<i32>,
}

// Set default values. If you don't implement `Default` it wont work.
impl Default for UserFactory {
    fn default() -> UserFactory {
        UserFactory {
            name: "Bob".into(),
            age: 30,
            country_id: None,
        }
    }
}

// FIXME this is dummy code to allow compilation
// Next step is to get it working using hand-written code
// Then translate into a macro

trait CountryAssociation {
    fn country_id(&self) -> i32;
}

impl CountryAssociation for CountryFactory {
    fn country_id(&self) -> i32 {
        dbg!(self);
        unimplemented!();
    }
}

impl CountryAssociation for Country {
    fn country_id(&self) -> i32 {
        self.id
    }
}

impl UserFactory {
    fn country(mut self, association: &CountryAssociation) -> Self {
        self.country_id = Some(association.country_id());
        self
    }
}
// END FIXME

// Setup the model. We have to implement `Identifiable`.
#[derive(Queryable, Identifiable)]
pub struct Country {
    pub id: i32,
    pub name: String,
}

// On a normal Diesel `Insertable` you can derive `Factory`
#[derive(Insertable, Factory, Debug)]
#[table_name = "countrys"]
// And specify which model type the factory is for
#[factory_model(Country)]
pub struct CountryFactory {
    name: String,
}

// Set default values. If you don't implement `Default` it wont work.
impl Default for CountryFactory {
    fn default() -> CountryFactory {
        CountryFactory { name: "Usa".into() }
    }
}

#[test]
fn creating_user() {
    let con = setup();

    let bob = UserFactory::default().insert(&con);
    let alice = UserFactory::default().name("Alice").insert(&con);

    assert_eq!(bob.name, "Bob");
    assert_eq!(alice.name, "Alice");
    assert_ne!(alice.id, bob.id);

    assert_eq!(find_user_by_id(bob.id, &con).name, "Bob");
    assert_eq!(find_user_by_id(alice.id, &con).name, "Alice");
}
#[test]
fn creating_user_and_country_with_literal() {
    let con = setup();
    let country = CountryFactory::default().name("USA").insert(&con);

    let alice = UserFactory::default()
        .name("Alice")
        .country(&country)
        .insert(&con);

    let alice_db = find_user_by_id(alice.id, &con);
    assert_eq!(alice_db.name, "Alice");
    assert_eq!(
        find_country_by_id(alice_db.country_id.unwrap(), &con).name,
        "USA"
    );
}
#[test]
fn creating_user_and_country_with_builder() {
    let con = setup();

    let alice = UserFactory::default()
        .name("Alice")
        .country(&CountryFactory::default().name("USA"))
        .insert(&con);

    let alice_db = find_user_by_id(alice.id, &con);
    assert_eq!(alice_db.name, "Alice");
    assert_eq!(
        find_country_by_id(alice_db.country_id.unwrap(), &con).name,
        "USA"
    );
}

#[test]
fn create_two_users_with_the_same_country() {
    use self::countrys;
    use diesel::dsl::count_star;
    let con = setup();

    let country = CountryFactory::default().name("USA").insert(&con);

    let bob = UserFactory::default()
        .name("Bob")
        .country(&country)
        .insert(&con);

    let alice = UserFactory::default()
        .name("Alice")
        .country(&country)
        .insert(&con);

    assert_eq!(
        find_country_by_id(bob.country_id.unwrap(), &con).name,
        "USA"
    );
    assert_eq!(bob.country_id, alice.country_id);
    assert_eq!(
        find_country_by_id(bob.country_id.unwrap(), &con).id,
        find_country_by_id(alice.country_id.unwrap(), &con).id
    );
    assert_eq!(Ok(1), countrys::table.select(count_star()).first(&con));
}

#[test]
fn create_two_users_with_distinct_countrys_from_the_same_builder() {
    use self::countrys;
    use diesel::dsl::count_star;
    let con = setup();

    let country_factory = CountryFactory::default().name("USA");

    let bob = UserFactory::default()
        .name("Bob")
        .country(&country_factory)
        .insert(&con);

    // Imagine there are other useful properties set up on this builder
    let country_factory = country_factory.name("Canada");
    let alice = UserFactory::default()
        .name("Alice")
        .country(&country_factory)
        .insert(&con);

    assert_eq!(
        find_country_by_id(bob.country_id.unwrap(), &con).name,
        "USA"
    );
    assert_eq!(
        find_country_by_id(alice.country_id.unwrap(), &con).name,
        "Canada"
    );
    assert_ne!(bob.country_id, alice.country_id);
    assert_ne!(
        find_country_by_id(bob.country_id.unwrap(), &con).id,
        find_country_by_id(alice.country_id.unwrap(), &con).id
    );
    assert_eq!(Ok(2), countrys::table.select(count_star()).first(&con));
}

fn setup() -> PgConnection {
    let database_url = "postgres://localhost/diesel_factories_test";
    let con = PgConnection::establish(&database_url).unwrap();
    con.begin_test_transaction().unwrap();
    con
}

fn find_user_by_id(input: i32, con: &PgConnection) -> User {
    use self::users::dsl::*;
    users.filter(id.eq(input)).first::<User>(con).unwrap()
}

fn find_country_by_id(input: i32, con: &PgConnection) -> Country {
    use self::countrys::dsl::*;
    countrys
        .filter(id.eq(&input))
        .first::<Country>(con)
        .unwrap()
}
