#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory, MapModel};

mod schema {
    table! {
        users (identity) {
            identity -> Integer,
            name -> Text,
            age -> Integer,
            last_name -> Text,
            created_at -> Nullable<Timestamp>
        }
    }
}

#[derive(MapModel, Queryable, Clone)]
#[map_model(
    table = crate::schema::users,
)]
struct User {
    pub identity: i32,
    pub name: String,
    pub age: i32,
    pub last_name: String
}

#[derive(Clone, Factory)]
#[factory(
    model = User,
    table = crate::schema::users,
    connection = diesel::pg::PgConnection,
    id_name = identity,
    map_fields
)]
struct UserFactory {
    pub identity: i32,
    pub name: String,
    pub last_name: String
}

impl Default for UserFactory {
    fn default() -> Self {
        Self {
            identity: 32,
            name: format!("test"),
            last_name: format!("tset")
        }
    }
}

fn main() {}
