#[derive(Debug, Clone)]
pub enum Association<'a, M, F> {
    #[doc(hidden)]
    Model(&'a M),

    #[doc(hidden)]
    Factory(F),
}

impl<M, F: Default> Default for Association<'_, M, F> {
    fn default() -> Self {
        Association::Factory(F::default())
    }
}

impl<'a, M, F> Association<'a, M, F> {
    pub fn new_model(inner: &'a M) -> Self {
        Association::Model(inner)
    }

    pub fn new_factory(inner: F) -> Self {
        Association::Factory(inner)
    }
}

impl<M, F> Association<'_, M, F>
where
    F: Factory<Model = M> + Clone,
{
    pub fn insert_returning_id(&self, con: &F::Connection) -> F::Id {
        match self {
            Association::Model(model) => F::id_for_model(&model).clone(),
            Association::Factory(factory) => {
                let model = factory.clone().insert(con);
                F::id_for_model(&model).clone()
            }
        }
    }
}

pub trait Factory {
    type Model;
    type Id: Clone;
    type Connection;

    fn insert(self, con: &Self::Connection) -> Self::Model;

    fn id_for_model(model: &Self::Model) -> &Self::Id;
}
