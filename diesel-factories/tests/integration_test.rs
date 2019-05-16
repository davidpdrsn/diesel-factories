#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};
use schema::{countries, users};

mod schema {
    table! {
        users (id) {
            id -> Integer,
            name -> Text,
            age -> Integer,
            country_id -> Integer,
        }
    }

    table! {
        countries (id) {
            id -> Integer,
            name -> Text,
        }
    }
}

#[derive(Queryable, Clone)]
struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub country_id: i32,
}

#[derive(Queryable, Clone)]
struct Country {
    pub id: i32,
    pub name: String,
}

// -- factories ------------

#[derive(Clone)]
struct UserFactory<'a> {
    pub name: String,
    pub age: i32,
    pub country: Association<'a, Country, CountryFactory>,
}

impl<'a> Default for UserFactory<'a> {
    fn default() -> Self {
        Self {
            name: "Bob".into(),
            age: 30,
            country: Association::default(),
        }
    }
}

#[derive(Clone)]
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

// -- macro ----------------

impl<'a> Factory for UserFactory<'a> {
    type Model = User;
    type Id = i32;
    type Connection = PgConnection;

    fn insert(self, con: &Self::Connection) -> Self::Model {
        use crate::schema::users;
        use diesel::prelude::*;
        let values = (
            (users::name.eq(&self.name)),
            (users::age.eq(&self.age)),
            (users::country_id.eq(self.country.insert_returning_id(con))),
        );
        diesel::insert_into(users::table)
            .values(values)
            .get_result::<Self::Model>(con)
            .unwrap()
    }

    fn id_for_model(model: &Self::Model) -> &Self::Id {
        &model.id
    }
}

trait SetCountryOnUserFactory<T> {
    fn country(self, t: T) -> Self;
}

impl<'a> SetCountryOnUserFactory<&'a Country> for UserFactory<'a> {
    fn country(mut self, country: &'a Country) -> Self {
        self.country = Association::new_model(country);
        self
    }
}

impl<'a> SetCountryOnUserFactory<CountryFactory> for UserFactory<'a> {
    fn country(mut self, factory: CountryFactory) -> Self {
        self.country = Association::new_factory(factory);
        self
    }
}

impl CountryFactory {
    fn name<T: Into<String>>(mut self, t: T) -> Self {
        self.name = t.into();
        self
    }
}

impl Factory for CountryFactory {
    type Model = Country;
    type Id = i32;
    type Connection = PgConnection;

    fn insert(self, con: &Self::Connection) -> Self::Model {
        use crate::schema::countries;
        use diesel::prelude::*;
        let values = (countries::name.eq(&self.name));
        diesel::insert_into(countries::table)
            .values(values)
            .get_result::<Self::Model>(con)
            .unwrap()
    }

    fn id_for_model(model: &Self::Model) -> &Self::Id {
        &model.id
    }
}

#[test]
fn insert_one_user() {
    let con = setup();

    let user = UserFactory::default().insert(&con);

    assert_eq!(user.name, "Bob");
    assert_eq!(user.age, 30);
    assert_eq!(1, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

#[test]
fn overriding_country() {
    let con = setup();

    let bob = UserFactory::default()
        .country(CountryFactory::default().name("USA"))
        .insert(&con);

    let country = find_country_by_id(bob.country_id, &con);

    assert_eq!("USA", country.name);
    assert_eq!(1, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

#[test]
fn insert_two_users_sharing_country() {
    let con = setup();

    let country = CountryFactory::default().insert(&con);
    let bob = UserFactory::default().country(&country).insert(&con);
    let alice = UserFactory::default().country(&country).insert(&con);

    assert_eq!(bob.country_id, alice.country_id);
    assert_eq!(2, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

fn setup() -> PgConnection {
    let database_url = "postgres://localhost/diesel_factories_test";
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

fn find_country_by_id(input: i32, con: &PgConnection) -> Country {
    use crate::schema::countries::dsl::*;
    countries
        .filter(id.eq(&input))
        .first::<Country>(con)
        .unwrap()
}
