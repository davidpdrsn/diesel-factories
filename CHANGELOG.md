# Change Log

All user visible changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/), as described
for Rust libraries in [RFC #1105](https://github.com/rust-lang/rfcs/blob/master/text/1105-api-evolution.md)

## Unreleased

Code generation has been rewritten and should provide better error messages.

syn, quote, and proc-macro2 dependencies have been migrated to version 1.0.

Support customizing foreign key name with `#[factory(foreign_key_name = _)]` on association fields:

```rust
mod schema {
    table! {
        users (id) {
            id -> Integer,
            age -> Integer,
            country_id -> Integer,
        }
    }
}

#[derive(Clone, Factory)]
#[factory(model = User, table = crate::schema::users)]
struct UserFactory<'a> {
    #[factory(foreign_key_name = country_id)]
    pub country: Association<'a, Country, CountryFactory>,
}
```

Factories fields can now be more types including references and arrays ([#21](https://github.com/davidpdrsn/diesel-factories/issues/21)).

### Breaking changes

Arguments to the `#[factory]` attribute are no longer surrounded by quotes:

```rust
#[derive(Clone, Factory)]
#[factory(
    model = User,
    table = crate::schema::users,
    connection = diesel::pg::PgConnection,
    id = i32,
    id_name = id,
)]
struct UserFactory {
    // ...
}
```

## 1.0.1

- Add `id_name` attribute for customizing the name of the id column of your table. Previously this was hard coded to `id`.

## 1.0.0

No changes were made but the API is now considered stable.

## 0.1.2

- Support for using path names for models.

## 0.1.1

### Added

- Support creating factories for models that just have an `id` field.

## 0.1.0

Completely rewritten implementation with a much nicer API.

## 0.0.1

Initial release.
