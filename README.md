# Diesel factories

Test factories for Diesel.

Still very much work in progress.


To set up cargo fmt as a pre-commit hook in git:
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