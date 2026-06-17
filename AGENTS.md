OxIRI is a library to parse IRIs following RFC 3987.

It allows parsing IRIs (`Iri` struct) and IRI references (`IriRef` struct) without normalizing them (`parse` and `parse_unchecked` methods)
or to resolve relative IRIs (`resolve`, `resolve_unchecked` method).

Use regular cargo tooling: `cargo +nightly fmt` for formatting, `cargo +nightly clippy` for linting, `cargo test` to run tests and `cargo bench`for benchmarking.

To check the correctness of the parser and resolver and relativization algorithms you can run for a few minutes the fuzzers:
- `cargo fuzz run -s none parse` for parsing
- `cargo fuzz run -s none resolve` for relative IRI resolution
- `cargo fuzz run -s none relativize` for IRI relativization
