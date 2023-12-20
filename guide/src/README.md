# Introduction

`more-di-axum` is a crate which provides dependency injection (DI) extensions for the [axum](https://crates.io/crates/axum)
web framework. Any `trait` or `struct` can be used as an injected service.

`axum` provides a [dependency injection example](https://github.com/tokio-rs/axum/blob/main/examples/error-handling-and-dependency-injection/src/main.rs); however, it is very limited. `axum` does not have nor provide a
fully-fledged DI framework. _State_ in `axum` must support `Clone` and is copied many times within the pipeline. In particular,
the native _State_ model does not intrinsically support a _scoped_ (e.g. per-request) lifetime. This is a limitation of
`Clone`. A _state_ can be wrapped in `Arc` as a _singleton_; otherwise, it is _transient_. `more-di-axum` brings full support
for various lifetimes by layering over the [more-di](https://crates.io/crates/more-di) library and makes them ergonomic to
consume within `axum`. Since `more-di` is a complete DI framework, swapping out dependency registration in different contexts,
such as testing, is trivial.

## Contributing

`more-di-axum` is free and open source. You can find the source code on [GitHub](https://github.com/commonsensesoftware/more-rs-di-axum)
and issues and feature requests can be posted on the [GitHub issue tracker](https://github.com/commonsensesoftware/more-rs-di-axum/issues).
`more-di-axum` relies on the community to fix bugs and add features: if you'd like to contribute, please read the
[CONTRIBUTING](https://github.com/commonsensesoftware/more-rs-di-axum/blob/main/CONTRIBUTING.md) guide and consider opening
a [pull request](https://github.com/commonsensesoftware/more-rs-di-axum/pulls).

## License

This project is licensed under the [MIT](https://github.com/commonsensesoftware/more-rs-di-axum/blob/main/LICENSE) license.