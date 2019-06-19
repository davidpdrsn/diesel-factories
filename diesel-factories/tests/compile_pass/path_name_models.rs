#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod models {
    pub mod schema {
        table! {
            users (id) {
                id -> Integer,
                name -> Text,
                age -> Integer,
                country_id -> Nullable<Integer>,
                home_city_id -> Nullable<Integer>,
                current_city_id -> Nullable<Integer>,
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
    pub struct User {
        pub id: i32,
        pub name: String,
        pub age: i32,
        pub country_id: Option<i32>,
        pub home_city_id: Option<i32>,
        pub current_city_id: Option<i32>,
    }

    #[derive(Clone, Queryable)]
    pub struct City {
        pub id: i32,
        pub name: String,
        pub country_id: i32,
    }

    #[derive(Clone, Queryable)]
    pub struct Country {
        pub id: i32,
        pub name: String,
    }
}

#[derive(Clone, Factory)]
#[factory(
    model = "crate::models::Country",
    table = "crate::models::schema::countries"
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

#[derive(Clone, Factory)]
#[factory(model = "crate::models::City", table = "crate::models::schema::cities")]
struct CityFactory<'a> {
    pub name: String,
    pub country: Association<'a, crate::models::Country, CountryFactory>,
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
