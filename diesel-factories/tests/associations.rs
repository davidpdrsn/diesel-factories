#[macro_use]
extern crate diesel;
extern crate diesel_factories;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_factories::InsertFactory;

// Tell Diesel what our schema is
table! {
    users (id) {
        id -> Integer,
        name -> Text,
        age -> Integer,
        country_id -> Nullable<Integer>,
    }
}

table! {
    countrys (id) {
        id -> Integer,
        name -> Text,
    }
}

// Setup the model. We have to implement `Identifiable`.
#[derive(Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub country_id: Option<i32>,
}

// On a normal Diesel `Insertable` you can derive `Factory`
pub struct UserFactory<'a> {
    name: String,
    age: i32,
    country_id: Option<i32>,
    connection: &'a PgConnection,
}

impl<'a> UserFactory<'a> {
    fn country(mut self, association: &CountryAssociation) -> UserFactory<'a> {
        self.country_id = Some(association.country_id());
        self
    }

    // Set default values here
    fn new(connection_in: &'a PgConnection) -> UserFactory<'a> {
        UserFactory {
            name: "Bob".into(),
            age: 30,
            country_id: None,
            connection: connection_in,
        }
    }

    fn name(mut self, new_value: &str) -> UserFactory<'a> {
        self.name = new_value.to_string();
        self
    }

    fn insert(self) -> User {
        use self::users::dsl::*;
        let res = diesel::insert_into(users)
            .values((
                (name.eq(&self.name)),
                age.eq(&self.age),
                country_id.eq(&self.country_id),
            ))
            .get_result::<User>(self.connection);

        match res {
            Ok(x) => x,
            Err(err) => panic!("{}", err),
        }
    }
}

trait CountryAssociation {
    fn country_id(&self) -> i32;
}

impl<'a> CountryAssociation for CountryFactory<'a> {
    fn country_id(&self) -> i32 {
        let country = self.insert();
        country.id
    }
}

impl CountryAssociation for Country {
    fn country_id(&self) -> i32 {
        self.id
    }
}

#[derive(Queryable, Identifiable)]
pub struct Country {
    pub id: i32,
    pub name: String,
}

pub struct CountryFactory<'a> {
    name: String,
    connection: &'a PgConnection,
}

impl<'a> CountryFactory<'a> {
    // Set default values here
    fn new(connection_in: &'a PgConnection) -> CountryFactory<'a> {
        CountryFactory {
            name: "USA".into(),
            connection: connection_in,
        }
    }

    fn name(mut self, new_value: &str) -> CountryFactory<'a> {
        self.name = new_value.to_string();
        self
    }

    fn insert(&self) -> Country {
        use self::countrys::dsl::*;
        let res = diesel::insert_into(countrys)
            .values(name.eq(&self.name))
            .get_result::<Country>(self.connection);

        match res {
            Ok(x) => x,
            Err(err) => panic!("{}", err),
        }
    }
}

#[test]
fn creating_user() {
    let con = setup();
    let alice = UserFactory::new(&con).name("Alice").insert();
    let bob = UserFactory::new(&con).name("Bob").insert();

    assert_eq!(bob.name, "Bob");
    assert_eq!(alice.name, "Alice");
    assert_ne!(alice.id, bob.id);

    assert_eq!(find_user_by_id(bob.id, &con).name, "Bob");
    assert_eq!(find_user_by_id(alice.id, &con).name, "Alice");
}
#[test]
fn creating_user_and_country_with_literal() {
    let con = setup();
    let country = CountryFactory::new(&con).name("USA").insert();

    let alice = UserFactory::new(&con)
        .name("Alice")
        .country(&country)
        .insert();

    let alice_db = find_user_by_id(alice.id, &con);

    assert_eq!(alice_db.name, "Alice");
    assert_eq!(
        find_country_by_id(alice_db.country_id.unwrap(), &con).name,
        "USA"
    );
}
#[test]
fn creating_user_and_country_with_builder() {
    let con = setup();

    let alice = UserFactory::new(&con)
        .name("Alice")
        .country(&CountryFactory::new(&con).name("USA"))
        .insert();

    let alice_db = find_user_by_id(alice.id, &con);
    assert_eq!(alice_db.name, "Alice");
    assert_eq!(
        find_country_by_id(alice_db.country_id.unwrap(), &con).name,
        "USA"
    );
}

#[test]
fn create_two_users_with_the_same_country() {
    use self::countrys;
    use diesel::dsl::count_star;
    let con = setup();

    let country = CountryFactory::new(&con).name("USA").insert();

    let bob = UserFactory::new(&con)
        .name("Bob")
        .country(&country)
        .insert();

    let alice = UserFactory::new(&con)
        .name("Alice")
        .country(&country)
        .insert();

    assert_eq!(
        find_country_by_id(bob.country_id.unwrap(), &con).name,
        "USA"
    );
    assert_eq!(bob.country_id, alice.country_id);
    assert_eq!(
        find_country_by_id(bob.country_id.unwrap(), &con).id,
        find_country_by_id(alice.country_id.unwrap(), &con).id
    );
    assert_eq!(Ok(1), countrys::table.select(count_star()).first(&con));
}

#[test]
fn create_two_users_with_distinct_countrys_from_the_same_builder() {
    use self::countrys;
    use diesel::dsl::count_star;
    let con = setup();

    let country_factory = CountryFactory::new(&con).name("USA");

    let bob = UserFactory::new(&con)
        .name("Bob")
        .country(&country_factory)
        .insert();

    // Imagine there are other useful properties set up on this builder
    let country_factory = country_factory.name("Canada");
    let alice = UserFactory::new(&con)
        .name("Alice")
        .country(&country_factory)
        .insert();

    assert_eq!(
        find_country_by_id(bob.country_id.unwrap(), &con).name,
        "USA"
    );
    assert_eq!(
        find_country_by_id(alice.country_id.unwrap(), &con).name,
        "Canada"
    );
    assert_ne!(bob.country_id, alice.country_id);
    assert_ne!(
        find_country_by_id(bob.country_id.unwrap(), &con).id,
        find_country_by_id(alice.country_id.unwrap(), &con).id
    );
    assert_eq!(Ok(2), countrys::table.select(count_star()).first(&con));
}

fn setup() -> PgConnection {
    let database_url = "postgres://localhost/diesel_factories_test";
    let con = PgConnection::establish(&database_url).unwrap();
    con.begin_test_transaction().unwrap();
    con
}

fn find_user_by_id(input: i32, con: &PgConnection) -> User {
    use self::users::dsl::*;
    users.filter(id.eq(input)).first::<User>(con).unwrap()
}

fn find_country_by_id(input: i32, con: &PgConnection) -> Country {
    use self::countrys::dsl::*;
    countrys
        .filter(id.eq(&input))
        .first::<Country>(con)
        .unwrap()
}
