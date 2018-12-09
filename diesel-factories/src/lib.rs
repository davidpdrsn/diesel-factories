use diesel::associations::HasTable;
use diesel::backend::SupportsDefaultKeyword;
use diesel::backend::{Backend, SupportsReturningClause};
use diesel::connection::Connection;
use diesel::insertable::CanInsertInSingleQuery;
use diesel::prelude::*;
use diesel::query_builder::QueryFragment;
use diesel::sql_types::HasSqlType;
use std::default::Default;

pub trait NewFactory<T: Default> {
    fn factory() -> T {
        T::default()
    }
}

pub trait FactoryInsert {
    fn insert<Model, Con, DB>(self, con: &Con) -> Model
    where
        Self: Insertable<<Model as HasTable>::Table>,
        <Self as Insertable<<Model as HasTable>::Table>>::Values:
            CanInsertInSingleQuery<DB> + QueryFragment<DB>,
        Con: Connection<Backend = DB>,
        DB: 'static
            + Backend<RawValue = [u8]>
            + SupportsReturningClause
            + HasSqlType<<<<Model as HasTable>::Table as Table>::AllColumns as Expression>::SqlType>,
        Model: HasTable
            + Queryable<
                <<<Model as HasTable>::Table as Table>::AllColumns as Expression>::SqlType,
                DB,
            >,
        <<Model as HasTable>::Table as Table>::AllColumns: QueryFragment<DB>,
        <<Model as HasTable>::Table as QuerySource>::FromClause: QueryFragment<DB>;
}

impl<Factory> FactoryInsert for Factory {
    fn insert<Model, Con, DB>(self, con: &Con) -> Model
    where
        Self: Insertable<<Model as HasTable>::Table>,
        <Self as Insertable<<Model as HasTable>::Table>>::Values:
            CanInsertInSingleQuery<DB> + QueryFragment<DB>,
        Con: Connection<Backend = DB>,
        DB: 'static
            + Backend<RawValue = [u8]>
            + SupportsReturningClause
            + HasSqlType<<<<Model as HasTable>::Table as Table>::AllColumns as Expression>::SqlType>,
        Model: HasTable
            + Queryable<
                <<<Model as HasTable>::Table as Table>::AllColumns as Expression>::SqlType,
                DB,
            >,
        <<Model as HasTable>::Table as Table>::AllColumns: QueryFragment<DB>,
        <<Model as HasTable>::Table as QuerySource>::FromClause: QueryFragment<DB>,
    {
        diesel::insert_into(Model::table())
            .values(self)
            .get_result::<Model>(con)
            .expect("Insert failed")
    }
}
