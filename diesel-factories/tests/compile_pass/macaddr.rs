#![allow(proc_macro_derive_resolution_fallback, unused_imports)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod schema {
    table! {
        devices {
            id -> Integer,
            macaddr -> MacAddr,
        }
    }
}

#[derive(Queryable, Clone)]
struct Device {
    pub id: i32,
    pub macaddr: [u8; 6],
}

#[derive(Clone, Factory)]
#[factory(
    model = Device,
    table = crate::schema::devices,
    connection = diesel::pg::PgConnection
)]
struct DeviceFactory {
    macaddr: [u8; 6],
}

impl Default for DeviceFactory {
    fn default() -> Self {
        Self {
            macaddr: [0; 6],
        }
    }
}

fn main() {}
