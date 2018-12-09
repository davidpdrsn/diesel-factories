//! ```
//! #[macro_use]
//! extern crate diesel;
//!
//! use diesel::prelude::*;
//! use diesel::pg::PgConnection;
//! use diesel_factories::{Factory, InsertFactory, DefaultFactory};
//!
//! // Tell Diesel what our schema is
//! table! {
//!     users (id) {
//!         id -> Integer,
//!         name -> Text,
//!         age -> Integer,
//!     }
//! }
//!
//! // Setup the model. We have to implement `Identifiable`.
//! #[derive(Queryable, Identifiable)]
//! pub struct User {
//!     pub id: i32,
//!     pub name: String,
//!     pub age: i32,
//! }
//!
//! // On a normal Diesel `Insertable` you can derive `Factory`
//! #[derive(Insertable, Factory)]
//! #[table_name = "users"]
//! // And specify which model type the factory is for
//! #[factory_model(User)]
//! pub struct UserFactory {
//!     name: String,
//!     age: i32,
//! }
//!
//! // Set default values. If you don't implement `Default` it wont work.
//! impl Default for UserFactory {
//!     fn default() -> UserFactory {
//!         UserFactory {
//!             name: "Bob".into(),
//!             age: 30,
//!         }
//!     }
//! }
//!
//! fn main() {
//!     use self::users::dsl::*;
//!
//!     // Connect to the database
//!     let database_url = "postgres://localhost/diesel_factories_test";
//!     let con = PgConnection::establish(&database_url).unwrap();
//!     # con.begin_test_transaction();
//!
//!     // Create a new user using our factory, overriding the default name
//!     let user = User::default_factory().name("Alice").insert(&con);
//!     assert_eq!("Alice", user.name);
//!     assert_eq!(30, user.age);
//!
//!     // Verifing that the user is in fact in the database
//!     let user_from_db = users
//!             .filter(id.eq(user.id))
//!             .first::<User>(&con)
//!             .unwrap();
//!     assert_eq!("Alice", user_from_db.name);
//!     assert_eq!(30, user_from_db.age);
//! }
//! ```

use diesel::backend::Backend;
use diesel::backend::SupportsDefaultKeyword;
use diesel::backend::SupportsReturningClause;
use diesel::connection::Connection;
use diesel::sql_types::HasSqlType;
use std::default::Default;
use diesel::pg::Pg;

pub use diesel_factories_code_gen::Factory;

/// Indicate which factory is the default for a given model type.
///
/// This trait is implemented automatically when you use `#[derive(Factory)]`.
///
/// impl DefaultFactory<#factory_name> for #model_name {}
/// ```
/// # #[macro_use]
/// # extern crate diesel;
/// # use diesel::prelude::*;
/// # use diesel::pg::PgConnection;
/// # use diesel_factories::{Factory, InsertFactory, DefaultFactory};
/// # table! {
/// #     users (id) { id -> Integer, name -> Text, age -> Integer, }
/// # }
/// # #[derive(Queryable, Identifiable)]
/// # pub struct User {
/// #     pub id: i32,
/// #     pub name: String,
/// #     pub age: i32,
/// # }
/// # #[derive(Insertable)]
/// # #[table_name = "users"]
/// # pub struct UserFactory {
/// #     name: String,
/// #     age: i32,
/// # }
/// # impl Default for UserFactory {
/// #     fn default() -> UserFactory {
/// #         UserFactory { name: "Bob".into(), age: 30, }
/// #     }
/// # }
/// impl DefaultFactory<UserFactory> for User {}
///
/// # fn main() {
/// let factory: UserFactory = User::default_factory();
/// # }
/// ```
pub trait DefaultFactory<T: Default> {
    fn default_factory() -> T {
        T::default()
    }
}

/// Method for inserting a factory object into the database.
pub trait InsertFactory<T> {
    /// Perform the insert and return the model.
    ///
    /// Will panic if there was a database error. That should be fine since you want to fail fast
    /// in tests when something goes wrong.
    fn insert<Con>(self, con: &Con) -> T
    where
        Con: Connection<Backend = Pg>;
}
