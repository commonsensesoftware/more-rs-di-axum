# More DI for Axum &emsp; ![CI][ci-badge] [![Crates.io][crates-badge]][crates-url] [![MIT licensed][mit-badge]][mit-url] 

[crates-badge]: https://img.shields.io/crates/v/more-di-axum.svg
[crates-url]: https://crates.io/crates/more-di-axum
[mit-badge]: https://img.shields.io/badge/license-MIT-blueviolet.svg
[mit-url]: https://github.com/commonsensesoftware/more-rs-di-axum/blob/main/LICENSE
[ci-badge]: https://github.com/commonsensesoftware/more-rs-di-axum/actions/workflows/ci.yml/badge.svg

More DI is a dependency injection (DI) library for Rust. This library provides additional DI extensions for the
[axum](https://crates.io/crates/axum) web framework.

You may be looking for:

- [User Guide](https://commonsensesoftware.github.io/more-rs-di-axum)
- [API Documentation](https://docs.rs/more-di-axum)
- [Release Notes](https://github.com/commonsensesoftware/more-rs-di-axum/releases)

## Dependency Injection in Axum

Consider the following structure.

```rust
use di::*;

#[injectable]
struct Person;

impl Person {
    fn speak(&self) -> &str {
        "Hello world!"
    }
}
```

This information can now be composed into a web application:

```rust
use crate::*;
use di::*;
use di_axum::*;

async fn say_hello(Inject(person): Inject<Person>) -> String {
    person.speak().to_owned()
}

#[tokio::main]
async fn main() {
    let provider = ServiceCollection::new()
        .add(Person::scoped())
        .build_provider()
        .unwrap();

    let app = Router::new()
        .route("/hello", get(say_hello))
        .with_provider(provider);

    let listener = TcpListener::bind("127.0.0.1:5000").await.unwrap();

    println!("Now listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/more-rs-di-axum/blob/main/LICENSE