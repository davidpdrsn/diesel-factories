#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod schema {
    table! {
        users (id) {
            id -> Integer,
            country_identifier -> Integer,
        }
    }

    table! {
        countries (id) {
            id -> Integer,
            #[sql_name = "type"]
            type_ -> Integer,
        }
    }
}

#[derive(Queryable, Clone)]
struct User {
    pub id: i32,
    pub country_identifier: i32,
}

#[derive(Clone, Factory)]
#[factory(
    model = User,
    table = crate::schema::users,
    connection = diesel::pg::PgConnection,
)]
struct UserFactory<'a> {
    #[factory(foreign_key_name = country_identifier)]
    pub country: Association<'a, Country, CountryFactory>,
}

impl Default for UserFactory<'_> {
    fn default() -> Self {
        Self {
            country: Default::default(),
        }
    }
}

#[derive(Queryable, Clone)]
struct Country {
    pub id: i32,
    pub r#type: i32,
}

#[derive(Clone, Factory)]
#[factory(
    model = Country,
    table = crate::schema::countries,
)]
struct CountryFactory {
    pub r#type: i32,
}

impl Default for CountryFactory {
    fn default() -> Self {
        Self {
            r#type: 0,
        }
    }
}

fn main() {
    let country = Country { id: 1, r#type: 0 };

    UserFactory::default()
        .country(CountryFactory::default())
        .country(&country);
}
