#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

table! {
    users {
        id -> Integer,
        slug -> Text,
    }
}

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
pub struct User {
    pub id: i32,
    pub slug: String,
}

#[derive(Clone, Factory)]
#[factory(
    model = User,
    table = users
)]
pub struct UserFactory<'a> {
    pub slug: &'a str,
}

fn main() {}
