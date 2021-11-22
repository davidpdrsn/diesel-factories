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
        countries (identity) {
            identity -> Integer,
            name -> Text,
        }
    }

    table! {
        cities (id) {
            id -> Integer,
            name -> Text,
            team_association -> Text,
            association_label -> Text,
            country_id -> Integer,
        }
    }

    table! {
        visited_cities (user_id, city_id) {
            user_id -> Integer,
            city_id -> Integer,
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

#[derive(Queryable, Clone)]
struct City {
    pub id: i32,
    pub name: String,
    pub team_association: String,
    pub association_label: String,
    pub country_id: i32,
}

#[derive(Clone, Factory)]
#[factory(model = City, table = crate::schema::cities)]
struct CityFactory<'b> {
    pub name: String,
    pub team_association: String,
    pub association_label: String,
    pub country: Association<'b, Country, CountryFactory>,
}

impl<'b> Default for CityFactory<'b> {
    fn default() -> Self {
        Self {
            name: "Copenhagen".into(),
            team_association: "teamfive".into(),
            association_label: "thebest".into(),
            country: Association::default(),
        }
    }
}

#[derive(Queryable, Clone)]
struct VisitedCity {
    pub user_id: i32,
    pub city_id: i32,
}


#[derive(Clone, Factory)]
#[factory(model = VisitedCity, table = crate::schema::visited_cities, no_id)]
struct VisitedCityFactory<'b> {
    pub user: Association<'b, User, UserFactory<'b>>,
    pub city: Association<'b, City, CityFactory<'b>>,
}

impl<'b> Default for VisitedCityFactory<'b> {
    fn default() -> Self {
        Self {
            user: Association::default(),
            city: Association::default(),
        }
    }
}

fn main() {
    let country = Country { id: 1, name: "Denmark".into() };

    let user_factory = UserFactory::default();

    let city_one = CityFactory::default()
        .country(&country);
    let city_two = CityFactory::default()
        .name("Another city")
        .country(&country);

    let visited_city_one = VisitedCityFactory::default()
        .user(user_factory.clone())
        .city(city_one.clone());

    let visited_city_two = VisitedCityFactory::default()
        .user(user_factory.clone())
        .city(city_two.clone());


    assert_eq!(
        visited_city_one.city.factory().unwrap().name,
        city_one.name
    );

    assert_eq!(
        visited_city_one.user.factory().unwrap().email,
        user_factory.email
    );


    assert_eq!(
        visited_city_two.city.factory().unwrap().name,
        city_two.name
    );

    assert_eq!(
        visited_city_two.user.factory().unwrap().email,
        user_factory.email
    );
}
