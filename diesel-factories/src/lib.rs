//! TODO

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

use lazy_static::lazy_static;
use std::sync::atomic::{AtomicUsize, Ordering};

pub use diesel_factories_code_gen::Factory;

/// TODO
#[derive(Debug, Clone)]
pub enum Association<'a, Model, Factory> {
    #[doc(hidden)]
    Model(&'a Model),

    #[doc(hidden)]
    Factory(Factory),
}

impl<Model, Factory: Default> Default for Association<'_, Model, Factory> {
    fn default() -> Self {
        Association::Factory(Factory::default())
    }
}

impl<'a, Model, Factory> Association<'a, Model, Factory> {
    #[doc(hidden)]
    pub fn new_model(inner: &'a Model) -> Self {
        Association::Model(inner)
    }

    #[doc(hidden)]
    pub fn new_factory(inner: Factory) -> Self {
        Association::Factory(inner)
    }
}

impl<Model, Factory> Association<'_, Model, Factory>
where
    Factory: FactoryMethods<Model = Model> + Clone,
{
    #[doc(hidden)]
    pub fn insert_returning_id(&self, con: &Factory::Connection) -> Factory::Id {
        match self {
            Association::Model(model) => Factory::id_for_model(&model).clone(),
            Association::Factory(factory) => {
                let model = factory.clone().insert(con);
                Factory::id_for_model(&model).clone()
            }
        }
    }
}

/// TODO
pub trait FactoryMethods {
    /// TODO
    type Model;
    /// TODO
    type Id: Clone;
    /// TODO
    type Connection;

    /// TODO
    fn insert(self, con: &Self::Connection) -> Self::Model;

    /// TODO
    fn id_for_model(model: &Self::Model) -> &Self::Id;
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
