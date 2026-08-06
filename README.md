# JSender: JSend in Pure Rust

![Crates.io Version](https://img.shields.io/crates/v/jsender)
![Crates.io License](https://img.shields.io/crates/l/jsender)
![docs.rs](https://img.shields.io/docsrs/jsender)

Pure Rust implementation of the [JSend specification](https://github.com/omniti-labs/jsend) for JSON response bodies.

[Documentation](https://docs.rs/jsender/)

# About

This crate provides constructors for **JSend** `success`, `fail`, and `error` response bodies. Each response is paired with the [StatusCode](https://docs.rs/http/latest/http/status/struct.StatusCode.html) it should be served with, which is applied when the response is serialised into a [Response](https://docs.rs/http/latest/http/response/struct.Response.html). Building on the `http` crate keeps response construction decoupled from any particular web framework, so the crate can sit in front of any server stack that uses the `http::Response` type.

# Minimum Supported Rust Version

Rust **1.85.1** or higher.

# License

Licensed under the [MIT License](https://opensource.org/license/MIT).
