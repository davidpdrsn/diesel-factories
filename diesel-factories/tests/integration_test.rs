#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};
use std::env;

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
    pub country_id: Option<i32>,
    pub home_city_id: Option<i32>,
    pub current_city_id: Option<i32>,
}

#[derive(Queryable, Clone)]
struct Country {
    pub identity: i32,
    pub name: String,
}

#[derive(Queryable, Clone)]
struct City {
    pub id: i32,
    pub name: String,
    pub team_association: String,
    pub association_label: String,
    pub country_id: i32,
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
#[factory(
    model = Country,
    table = crate::schema::countries,
    id_name = identity
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

#[test]
fn insert_one_user() {
    let con = setup();

    let user = UserFactory::default().name("Alice").insert(&con);

    assert_eq!(user.name, "Alice");
    assert_eq!(user.age, 30);
    assert_eq!(1, count_users(&con));
    assert_eq!(0, count_countries(&con));
}

#[test]
fn overriding_country() {
    let con = setup();

    let bob = UserFactory::default()
        .country(Some(CountryFactory::default().name("USA")))
        .insert(&con);

    let country = find_country_by_id(bob.country_id.unwrap(), &con);

    assert_eq!("USA", country.name);
    assert_eq!(1, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

#[test]
fn insert_two_users_sharing_country() {
    let con = setup();

    let country = CountryFactory::default().insert(&con);
    let bob = UserFactory::default().country(Some(&country)).insert(&con);
    let alice = UserFactory::default().country(Some(&country)).insert(&con);

    assert_eq!(bob.country_id, alice.country_id);
    assert_eq!(2, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

#[test]
fn insert_visited_cities() {
    let con = setup();

    let country = CountryFactory::default().insert(&con);
    let user = UserFactory::default().country(Some(&country)).insert(&con);
    let city_one = CityFactory::default().country(&country).insert(&con);
    let city_two = CityFactory::default().country(&country).insert(&con);

    let visited_city_one = VisitedCityFactory::default().city(&city_one).user(&user).insert(&con);
    let visited_city_two = VisitedCityFactory::default().city(&city_two).user(&user).insert(&con);

    assert_eq!(user.country_id, Some(country.identity));
    assert_eq!(1, count_users(&con));
    assert_eq!(1, count_countries(&con));
    assert_eq!(2, count_cities(&con));
    assert_eq!(2, count_visited_cities(&con));
    assert_eq!(visited_city_one.user_id, user.id);
    assert_eq!(visited_city_one.city_id, city_one.id);
    assert_eq!(visited_city_two.user_id, user.id);
    assert_eq!(visited_city_two.city_id, city_two.id);
}

#[test]
fn visited_cities_build_whole_tree() {
    let con = setup();

    VisitedCityFactory::default().insert(&con);

    assert_eq!(1, count_users(&con));
    assert_eq!(1, count_countries(&con));
    assert_eq!(1, count_cities(&con));
    assert_eq!(1, count_visited_cities(&con));
}

fn setup() -> PgConnection {
    let pg_host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let pg_port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
    let pg_password = env::var("POSTGRES_PASSWORD").ok();

    let auth = if let Some(pg_password) = pg_password {
        format!("postgres:{}@", pg_password)
    } else {
        String::new()
    };

    let database_url = format!(
        "postgres://{auth}{host}:{port}/diesel_factories_test",
        auth = auth,
        host = pg_host,
        port = pg_port
    );
    let con = PgConnection::establish(&database_url).unwrap();
    con.begin_test_transaction().unwrap();
    con
}

fn count_users(con: &PgConnection) -> i64 {
    use crate::schema::users;
    use diesel::dsl::count_star;
    users::table.select(count_star()).first(con).unwrap()
}

fn count_countries(con: &PgConnection) -> i64 {
    use crate::schema::countries;
    use diesel::dsl::count_star;
    countries::table.select(count_star()).first(con).unwrap()
}

fn count_cities(con: &PgConnection) -> i64 {
    use crate::schema::cities;
    use diesel::dsl::count_star;
    cities::table.select(count_star()).first(con).unwrap()
}

fn count_visited_cities(con: &PgConnection) -> i64 {
    use crate::schema::visited_cities;
    use diesel::dsl::count_star;
    visited_cities::table.select(count_star()).first(con).unwrap()
}

fn find_country_by_id(input: i32, con: &PgConnection) -> Country {
    use crate::schema::countries::dsl::*;
    countries
        .filter(identity.eq(&input))
        .first::<Country>(con)
        .unwrap()
}
