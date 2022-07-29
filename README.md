# rocket_static_fs

[![Build Status](https://travis-ci.org/Ekranos/rocket_static_fs.svg?branch=master)](https://travis-ci.org/Ekranos/rocket_static_fs)

A simple static file server for Rust's rocket framework.

[Documentation](https://ekranos.github.io/rocket_static_fs/rocket_static_fs/)

## Features

- Basic HTTP caching via Last-Modified header
- `Content-Encoding` support (gzip and deflate)
- `Range` support (no multipart ranges yet)
- Support for multiple file backends:
  - LocalFileSystem => serve files from a local directory
  - EmbeddedFileSystem => serve files which are bundled into the binary
    - An example for that is documented on the EmbeddedFileSystem struct
  - You can add your own FileSystem implementations by implementing the fs::FileSystem trait
- Directory listing support (no defaulting to certain files right now (e.g. index.html))

## Todos

- Cache-Control header rules
- Directory listing default index file

## Suggestions / Contributions?

Submit an issue/PR. But in almost all cases it's better to first open
an issue before submitting a PR, so you don't waste your time implementing
a PR which may get rejected.

## Testing

Currently testing is a little bit weird. Before testing, you should `cargo run` once,
to create a test package for the `fs::embedded::Package` test.

Then you can test with `cargo test --all-features` since the `fs::embedded::Package` test is
currently behind a feature flag.
 
# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.
