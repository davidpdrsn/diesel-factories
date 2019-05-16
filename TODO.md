# TODO

- Support factory have multiple fields with the same type of association.

```rust
#[derive(Factory, Clone)]
#[factory(model = "User", table = "crate::schema::users")]
pub struct UserFactory<'a> {
    first: Option<Association<'a, Foo, FooFactory>>,
    second: Option<Association<'a, Foo, FooFactory>>,
         // ^^^^^^ Conflicting implementations of `Set{}On{}` trait
}
```
