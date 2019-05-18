# [diesel-factories](https://crates.io/crates/diesel-factories)

An implementation of the test factory pattern made to work with [Diesel](https://diesel.rs).

See [the documentation for more info](https://docs.rs/crate/diesel-factories).

## Development

To install the `cargo fmt` pre-commit githook:

```sh
git config core.hooksPath ./githooks
```

To run tests, you will need to create a database:

```sql
CREATE DATABASE diesel_factories_test;
```

And also run migrations:

```sh
cargo install diesel_cli --no-default-features --features postgres
diesel migration run --database-url postgresql://localhost:5432/diesel_factories_test
```

---

License: MIT
