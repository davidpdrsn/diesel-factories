#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory, FactoryMethods};

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

#[derive(Queryable, Clone)]
struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub country_id: Option<i32>,
}

#[derive(Queryable, Clone)]
struct Country {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Clone)]
struct City {
    pub id: i32,
    pub name: String,
    pub country_id: i32,
}

#[derive(Clone, Factory)]
#[factory(
    model = "User",
    table = "crate::schema::users",
    connection = "diesel::pg::PgConnection"
)]
struct UserFactory<'a> {
    pub name: String,
    pub age: i32,
    pub country: Option<Association<'a, Country, CountryFactory>>,
}

impl<'a> Default for UserFactory<'a> {
    fn default() -> Self {
        Self {
            name: "Bob".into(),
            age: 30,
            country: None,
        }
    }
}

#[derive(Clone, Factory)]
#[factory(model = "Country", table = "crate::schema::countries")]
struct CountryFactory {
    pub name: String,
}

impl Default for CountryFactory {
    fn default() -> Self {
        Self {
            name: "Denmark".into(),
        }
    }
}

#[derive(Clone, Factory)]
#[factory(model = "City", table = "crate::schema::cities")]
struct CityFactory<'a> {
    pub name: String,
    pub country: Association<'a, Country, CountryFactory>,
}

impl<'a> Default for CityFactory<'a> {
    fn default() -> Self {
        Self {
            name: "Copenhagen".into(),
            country: Association::default(),
        }
    }
}

#[test]
fn insert_one_user() {
    let con = setup();

    let user = UserFactory::default().name("Alice").insert(&con);

    assert_eq!(user.name, "Alice");
    assert_eq!(user.age, 30);
    assert_eq!(1, count_users(&con));
    assert_eq!(0, count_countries(&con));
}

#[test]
fn overriding_country() {
    let con = setup();

    let bob = UserFactory::default()
        .country(CountryFactory::default().name("USA"))
        .insert(&con);

    let country = find_country_by_id(bob.country_id.unwrap(), &con);

    assert_eq!("USA", country.name);
    assert_eq!(1, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

#[test]
fn insert_two_users_sharing_country() {
    let con = setup();

    let country = CountryFactory::default().insert(&con);
    let bob = UserFactory::default().country(&country).insert(&con);
    let alice = UserFactory::default().country(&country).insert(&con);

    assert_eq!(bob.country_id, alice.country_id);
    assert_eq!(2, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

fn setup() -> PgConnection {
    let database_url = "postgres://localhost/diesel_factories_test";
    let con = PgConnection::establish(&database_url).unwrap();
    con.begin_test_transaction().unwrap();
    con
}

fn count_users(con: &PgConnection) -> i64 {
    use crate::schema::users;
    use diesel::dsl::count_star;
    users::table.select(count_star()).first(con).unwrap()
}

fn count_countries(con: &PgConnection) -> i64 {
    use crate::schema::countries;
    use diesel::dsl::count_star;
    countries::table.select(count_star()).first(con).unwrap()
}

fn find_country_by_id(input: i32, con: &PgConnection) -> Country {
    use crate::schema::countries::dsl::*;
    countries
        .filter(id.eq(&input))
        .first::<Country>(con)
        .unwrap()
}
