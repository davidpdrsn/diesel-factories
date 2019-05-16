#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;
extern crate diesel_factories;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_factories::Association;
use diesel_factories::Factory;

// Tell Diesel what our schema is
mod schema {
    table! {
        users (id) {
            id -> Integer,
            name -> Text,
            age -> Integer,
            country_id -> Nullable<Integer>,
        }
    }

    table! {
        countries (id) {
            id -> Integer,
            name -> Text,
        }
    }

    table! {
        cities (id) {
            id -> Integer,
            name -> Text,
            country_id -> Integer,
        }
    }
}

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub country_id: Option<i32>,
}

#[derive(Factory)]
#[factory_model(User)]
#[table_name = "users"]
// TODO, refactor this to support the syntax below
// #[factory(table_name = users, model = User)]
pub struct UserFactory<'a> {
    name: String,
    age: i32,
    #[factory(model = Country, factory = CountryFactory)]
    country_id: Option<i32>,
    connection: &'a PgConnection,
}

impl<'a> UserFactory<'a> {
    fn new(connection_in: &'a PgConnection) -> UserFactory<'a> {
        UserFactory {
            name: "Bob".into(),
            age: 30,
            country_id: None,
            connection: connection_in,
        }
    }
}

#[derive(Queryable)]
pub struct Country {
    pub id: i32,
    pub name: String,
}

#[derive(Factory)]
#[factory_model(Country)]
#[table_name = "countries"]
pub struct CountryFactory<'a> {
    name: String,
    connection: &'a PgConnection,
}

impl<'a> CountryFactory<'a> {
    // Set default values here
    fn new(connection_in: &'a PgConnection) -> CountryFactory<'a> {
        CountryFactory {
            name: "USA".into(),
            connection: connection_in,
        }
    }
}

#[derive(Queryable)]
pub struct City {
    pub id: i32,
    pub name: String,
    pub country_id: i32,
}

#[derive(Factory)]
#[factory_model(City)]
#[table_name = "cities"]
pub struct CityFactory<'a> {
    name: String,
    #[factory(model = Country, factory = CountryFactory)]
    country_id: i32,
    connection: &'a PgConnection,
}

impl<'a> CityFactory<'a> {
    // Set default values here
    fn new(connection_in: &'a PgConnection) -> CityFactory<'a> {
        CityFactory {
            name: "New York".into(),
            country_id: CountryFactory::new(connection_in).insert().id,
            connection: connection_in,
        }
    }
}

#[test]
fn creating_user() {
    let con = setup();
    let alice = UserFactory::new(&con).name("Alice").insert();
    let bob = UserFactory::new(&con).name("Bob").insert();

    assert_eq!(bob.name, "Bob");
    assert_eq!(alice.name, "Alice");
    assert_ne!(alice.id, bob.id);

    assert_eq!(find_user_by_id(bob.id, &con).name, "Bob");
    assert_eq!(find_user_by_id(alice.id, &con).name, "Alice");
}
#[test]
fn creating_user_and_country_with_literal() {
    let con = setup();
    let country = CountryFactory::new(&con).name("USA").insert();

    let alice = UserFactory::new(&con)
        .name("Alice")
        .country(&country)
        .insert();

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

    let alice = UserFactory::new(&con)
        .name("Alice")
        .country(&CountryFactory::new(&con).name("USA"))
        .insert();

    let alice_db = find_user_by_id(alice.id, &con);
    assert_eq!(alice_db.name, "Alice");
    assert_eq!(
        find_country_by_id(alice_db.country_id.unwrap(), &con).name,
        "USA"
    );
}
#[test]
fn creating_city_and_country_with_literal() {
    let con = setup();
    let country = CountryFactory::new(&con).name("USA").insert();

    let ny = CityFactory::new(&con)
        .name("New York")
        .country(&country)
        .insert();

    let ny_db = find_city_by_id(ny.id, &con);

    assert_eq!(ny_db.name, "New York");
    assert_eq!(find_country_by_id(ny_db.country_id, &con).name, "USA");
}

#[test]
fn creating_city_and_country_with_builder() {
    let con = setup();

    let ny = CityFactory::new(&con)
        .name("New York")
        .country(&CountryFactory::new(&con).name("USA"))
        .insert();

    let ny_db = find_city_by_id(ny.id, &con);
    assert_eq!(ny_db.name, "New York");
    assert_eq!(find_country_by_id(ny_db.country_id, &con).name, "USA");
}

#[test]
fn does_not_create_too_many_cities() {
    use crate::schema::{cities, countries};
    use diesel::dsl::count_star;

    let con = setup();

    let ny = CityFactory::new(&con)
        .country(&CountryFactory::new(&con))
        .insert();

    assert_eq!(Ok(1), cities::table.select(count_star()).first(&con));
    assert_eq!(Ok(1), countries::table.select(count_star()).first(&con));
}

#[test]
fn create_two_users_with_the_same_country() {
    use crate::schema::countries;
    use diesel::dsl::count_star;
    let con = setup();

    let country = CountryFactory::new(&con).name("USA").insert();

    let bob = UserFactory::new(&con)
        .name("Bob")
        .country(&country)
        .insert();

    let alice = UserFactory::new(&con)
        .name("Alice")
        .country(&country)
        .insert();

    assert_eq!(
        find_country_by_id(bob.country_id.unwrap(), &con).name,
        "USA"
    );
    assert_eq!(bob.country_id, alice.country_id);
    assert_eq!(
        find_country_by_id(bob.country_id.unwrap(), &con).id,
        find_country_by_id(alice.country_id.unwrap(), &con).id
    );
    assert_eq!(Ok(1), countries::table.select(count_star()).first(&con));
}

#[test]
fn create_two_users_with_distinct_countries_from_the_same_builder() {
    use crate::schema::countries;
    use diesel::dsl::count_star;
    let con = setup();

    let country_factory = CountryFactory::new(&con).name("USA");

    let bob = UserFactory::new(&con)
        .name("Bob")
        .country(&country_factory)
        .insert();

    // Imagine there are other useful properties set up on this builder
    let country_factory = country_factory.name("Canada");
    let alice = UserFactory::new(&con)
        .name("Alice")
        .country(&country_factory)
        .insert();

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
    assert_eq!(Ok(2), countries::table.select(count_star()).first(&con));
}

fn setup() -> PgConnection {
    let database_url = "postgres://localhost/diesel_factories_test";
    let con = PgConnection::establish(&database_url).unwrap();
    con.begin_test_transaction().unwrap();
    con
}

fn find_user_by_id(input: i32, con: &PgConnection) -> User {
    use crate::schema::users::dsl::*;
    users.filter(id.eq(input)).first::<User>(con).unwrap()
}

fn find_city_by_id(input: i32, con: &PgConnection) -> City {
    use crate::schema::cities::dsl::*;
    cities.filter(id.eq(input)).first::<City>(con).unwrap()
}

fn find_country_by_id(input: i32, con: &PgConnection) -> Country {
    use crate::schema::countries::dsl::*;
    countries
        .filter(id.eq(&input))
        .first::<Country>(con)
        .unwrap()
}
