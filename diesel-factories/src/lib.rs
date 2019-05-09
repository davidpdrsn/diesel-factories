//! This libraries makes it straight forward to create [factories][] that work with [Diesel][].
//!
//! [factories]: https://thoughtbot.com/blog/why-factories
//! [Diesel]: https://diesel.rs
//!
//! # WIP
//!
//! It is still very much work in progress so expect breaking changes at any point.
//!
//!
//!
//! # What `#[derive(Factory)]` does
//!
//! Deriving `Factory` on a struct will do the following:
//!
//! 1. Implement [`DefaultFactory`][].
//! 1. Implement [`InsertFactory`][].
//! 1. Add methods to change the values set by your [`Default`][] implementation.
//!
//! For example this:
//!

extern crate diesel;

use diesel::connection::Connection;
use diesel::pg::Pg;
use lazy_static::lazy_static;
use std::{
    default::Default,
    sync::atomic::{AtomicUsize, Ordering},
};

pub use diesel_factories_code_gen::Factory;

/// Indicate which factory is the default for a given model type.
///
/// This trait is implemented automatically when you use `#[derive(Factory)]`.
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

lazy_static! {
    static ref SEQUENCE_COUNTER: AtomicUsize = { AtomicUsize::new(0) };
}

/// Utility function for generating unique ids or strings in factories.
/// Each time `sequence` gets called, the closure will receive a different number.
///
/// ```
/// use diesel_factories::sequence;
///
/// assert_ne!(
///     sequence(|i| format!("unique-string-{}", i)),
///     sequence(|i| format!("unique-string-{}", i)),
/// );
/// ```
pub fn sequence<T, F>(f: F) -> T
where
    F: Fn(usize) -> T,
{
    SEQUENCE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let count = SEQUENCE_COUNTER.load(Ordering::Relaxed);
    f(count)
}
