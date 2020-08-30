#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod schema {
    table! {
        users (id) {
            id -> Integer,
            name -> Text,
            age -> Integer,
            email -> Nullable<Text>,
            country_id -> Integer,
            home_country_id -> Nullable<Integer>,
        }
    }

    table! {
        countries (id) {
            id -> Integer,
            name -> Text,
        }
    }
}

#[derive(Queryable, Clone)]
struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub email: Option<String>,
    pub country_id: i32,
    pub home_country_id: Option<i32>,
}

#[derive(Clone, Factory)]
#[factory(
    model = User,
    table = crate::schema::users,
    connection = diesel::pg::PgConnection,
)]
struct UserFactory<'a> {
    pub name: String,
    pub age: i32,
    pub email: Option<String>,
    pub country: Association<'a, Country, CountryFactory>,
    pub home_country: Option<Association<'a, Country, CountryFactory>>,
}

impl Default for UserFactory<'_> {
    fn default() -> Self {
        Self {
            name: "Bob".into(),
            age: 30,
            email: None,
            country: Default::default(),
            home_country: Default::default(),
        }
    }
}

#[derive(Queryable, Clone)]
struct Country {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Factory)]
#[factory(
    model = Country,
    table = crate::schema::countries,
)]
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

fn main() {
    let country = Country { id: 1, name: "Denmark".into() };

    let user_factory = UserFactory::default()
        .name("Alice")
        .age(20)
        .email(Some("alice@exmple.com".into()))
        .country(CountryFactory::default())
        .country(&country)
        .home_country(Some(CountryFactory::default()))
        .home_country(Some(&country));

    assert_eq!(user_factory.name, "Alice");
    assert_eq!(user_factory.age, 20);
    assert_eq!(user_factory.email, Some("alice@exmple.com".into()));
}
