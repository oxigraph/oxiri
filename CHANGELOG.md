# Changelog

## [0.2.11] - 2025-01-19

### Changed

- Resolve: do not remove dot segment if a scheme or an authority is present for consistency with parse.

## [0.2.10] - 2025-01-17

### Changed

- Resolve: properly normalize relative paths starting with `.` or `..`.
- Relativize: make use of `.` and do not always use absolute path if there is a candidate relative path containing `/`.

## [0.2.9] - 2024-12-21

### Changed

- Relativize: avoid panic in case of shared UTF-8 prefix.

## [0.2.8] - 2024-10-19

### Changed

- Fixes relativize on IRIs with a scheme and nothing else.

## [0.2.7] - 2024-10-19

### Changed

- Fixes relative IRI resolution when there is no authority but a path starting with "/".

## [0.2.6] - 2024-10-18

### Changed

- Fixes relativize on hierarchical paths without authority and starting slash.

## [0.2.5] - 2024-10-03

### Added

- `Iri::relativize` to build a relative IRI from a base IRI and an absolute IRI.

## [0.2.4] - 2024-08-20

### Changed

- Makes IRI parsing a bit more strict to follow RFC 3987 more closely.
- Allow IP vFuture in authority.

## [0.2.3] - 2024-03-23

### Added

- `_unchecked` methods for faster parsing/resolving if the IRI is known to be valid.

## [0.2.2] - 2022-03-27

### Added

- `Iri` and `IriRef` now implement Serde `Serialize` and `Deserialize` traits if the `serde` crate is present.
  The serialization is a plain string.

## [0.2.1] - 2021-01-10

### Changed

- Fixes a regression in relative IRI parsing when there is only a host without a training slash. For example `foo` is
  now resolved against `http://example.org` as `http://example.org/foo` and not anymore as `http://example.orgfoo`.
- The validation of unicode character is now carefully following RFC 3987:
    - Some private use characters are not anymore allowed in path and fragment.
    - Some surrogates are not allowed anymore in query.
    - The range F900-FDEF is now allowed in path and fragment following the RFC.

## [0.2.0] - 2021-01-06

### Added

- `IriRef` type that provides the same API as `Iri` but for relative IRIs.
- `PartialOrder` implementations between `Iri`s with different container types.

### Changed

- Fixes path resolution: the resolver should return `tag:c-d` and not `tag:/c-d` when resolving `c-d` against `tag:a-b`.
- Relative IRIs are not anymore allowed to start with a column `:`.
- `iprivate` characters (`%xE000-F8FF / %xF0000-FFFFD / %x100000-10FFFD`) are not allowed anymore as part of the IRI
  query component following RFC 3987.

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