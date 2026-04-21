# Changelog

All notable changes to this crate are documented in this file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0]

- Bumped `mechanics-core` dep from `"0.2.2"` to `"0.3.0"`, following
  `mechanics-core`'s re-cut as `0.3.0`. See
  `mechanics-core/CHANGELOG.md [0.3.0]` for the underlying
  reasoning (`cargo-semver-checks` flagged the type-identity
  change from extracting schema types into the new
  `mechanics-config` crate as a breaking change under cargo's
  pre-1.0 semver rules). Call-site usage of re-exported types is
  preserved; the minor-digit bump here co-moves with
  `mechanics-core` so downstream consumers of `mechanics` opt in
  explicitly rather than silently crossing the type-identity
  boundary under a caret-range upgrade.

## [0.2.1]

Git history is the authoritative record for this and earlier
releases; future releases are documented going forward in this
file.
