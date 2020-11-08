#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod schema {
    table! {
        countries (identity) {
            identity -> Integer,
            name -> Text,
        }
    }

    table! {
        cities (identity) {
            identity -> Integer,
            name -> Text,
            country_identity -> Integer,
        }
    }
}

#[derive(Clone, Queryable)]
pub struct Country {
    pub identity: i32,
    pub name: String,
}

#[derive(Clone, Factory)]
#[factory(model = Country, table = schema::countries, id_name = identity)]
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

#[derive(Clone, Queryable)]
pub struct City {
    pub identity: i32,
    pub name: String,
    pub country_identity: i32,
}

#[derive(Clone, Factory)]
#[factory(model = City, table = schema::cities, id_name = identity)]
struct CityFactory<'a> {
    pub name: String,
    pub country: Association<'a, Country, CountryFactory>,
}

impl<'a> Default for CityFactory<'a> {
    fn default() -> Self {
        Self {
            name: String::new(),
            country: Association::default(),
        }
    }
}

fn main() {}
