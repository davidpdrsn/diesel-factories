#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod schema {
    table! {
        users (identity) {
            identity -> Integer,
        }
    }
}

#[derive(Queryable, Clone)]
struct User {
    pub identity: i32,
}

#[derive(Clone, Factory)]
#[factory(
    model = User,
    table = crate::schema::users,
    connection = diesel::pg::PgConnection,
    id_name = identity
)]
struct UserFactory {}

impl Default for UserFactory {
    fn default() -> Self {
        Self {}
    }
}

fn main() {}
