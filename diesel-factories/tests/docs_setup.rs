#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod schema {
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

#[derive(Clone, Queryable)]
struct City {
    pub id: i32,
    pub name: String,
    pub country_id: i32,
}

#[derive(Clone, Queryable)]
struct Country {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Factory)]
#[factory(model = Country, table = crate::schema::countries)]
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
