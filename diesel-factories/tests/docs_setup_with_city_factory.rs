#[macro_use]
extern crate diesel;

use diesel_factories::{Association, Factory};

mod schema {
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
    pub country_id: Option<i32>,
    pub home_city_id: Option<i32>,
    pub current_city_id: Option<i32>,
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

#[derive(Queryable, Clone)]
struct VisitedCity {
    pub user_id: i32,
    pub city_id: i32,
}


#[derive(Clone, Factory)]
#[factory(
model = User,
table = crate::schema::users,
connection = diesel::pg::PgConnection
)]
struct UserFactory<'a> {
    pub name: &'a str,
    pub age: i32,
    pub country: std::option::Option<diesel_factories::Association<'a, Country, CountryFactory>>,
    pub home_city: Option<diesel_factories::Association<'a, City, CityFactory<'a>>>,
    pub current_city: Option<Association<'a, City, CityFactory<'a>>>,
}

impl<'a> Default for UserFactory<'a> {
    fn default() -> Self {
        Self {
            name: "Bob",
            age: 30,
            country: None,
            home_city: None,
            current_city: None,
        }
    }
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

#[derive(Clone, Factory)]
#[factory(model = City, table = crate::schema::cities)]
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
