#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod schema {
    table! {
        users (id) {
            id -> Integer,
            age -> Integer,
            country_id -> Integer,
        }
    }

    table! {
        countries (id) {
            id -> Integer,
        }
    }
}

#[derive(Queryable, Clone)]
struct User {
    pub id: i32,
    pub age: i32,
    pub country_id: i32,
}

#[derive(Clone, Factory)]
#[factory(
    model = User,
    table = crate::schema::users,
    connection = diesel::pg::PgConnection,
)]
struct UserFactory<'a> {
    #[factory(foreign_key_name = not_allowed_here)]
    pub age: i32,
    pub country: Association<'a, Country, CountryFactory>,
}

impl Default for UserFactory<'_> {
    fn default() -> Self {
        Self {
            age: 30,
            country: Default::default(),
        }
    }
}

#[derive(Queryable, Clone)]
struct Country {
    pub id: i32,
}

#[derive(Clone, Factory)]
#[factory(
    model = Country,
    table = crate::schema::countries,
)]
struct CountryFactory {}

impl Default for CountryFactory {
    fn default() -> Self {
        Self {}
    }
}

fn main() {
    let country = Country { id: 1 };

    UserFactory::default()
        .country(CountryFactory::default())
        .country(&country);
}
