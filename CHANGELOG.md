# Changelog

## [0.1.1] - 2020-07-10

### Added
- Accessors for IRI scheme, authority, path, query and fragment.
- `PartialEq` and `From` implementations between `Iri` and some string types.
- `Iri` order and hash is now the same as `str`.
- `Borrow<Target=&str>` and `AsRef<Target=&str>` implementations for `Iri`.

### Changed
- Bug fix in the relative IRI resolution: some character were duplicated.

## [0.1.0] - 2020-05-01

### Added
- `Iri` struct with a parser and relative IRI resolution.