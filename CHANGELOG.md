# Changelog

All notable changes to this crate are documented in this file.

The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
this crate adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.3] - 2026-05-14

### Fixed
- `handle_h3_request` now takes
  `Request<mechanics_http_server::H3RequestBody>` instead of
  `Request<()>`, matching the `Http3Server::start` service
  contract that `mhs 0.1.3` switched to (`http_body::Body`
  streaming bodies). `mechanics 0.5.2` failed to compile
  against `mhs 0.1.3` because the type-signature update never
  made it into the consumer; the broken release shipped to
  crates.io and is superseded by this patch.

## [0.5.2] - 2026-05-14

### Added
- Added `MechanicsServer::run_tls_with_h3`, which starts an
  opportunistic HTTP/3 listener alongside the existing HTTPS server
  and advertises it with `Alt-Svc` on TLS responses.

## [0.5.1] - 2026-05-14

### Changed
- Internal Cargo.toml audit: `default-features = false` set on
  direct dependencies with explicit feature lists for what the
  crate actually uses. No behaviour change. (D24)

## [0.5.0] - 2026-05-13

Changed (breaking): bumped to `mechanics-core = "0.5"`, which renames
`ReqwestEndpointHttpClient` → `DefaultEndpointHttpClient` and switches
the default endpoint HTTP transport from `reqwest` to
`mechanics-http-client` (hyper-rustls + webpki-roots + aws-lc-rs).
Consumers that wired a custom `endpoint_http_client` need to update
the type name and the underlying client constructor.

## [0.4.2]

- Hardened HTTPS server-side TLS posture (only relevant when the
  `https` feature is enabled):
  - Removed AES128-class cipher suites from the
    `ServerConfig` cipher-suite list. Effective suites are now
    AES256-GCM and CHACHA20-POLY1305 only (TLS 1.3 and TLS 1.2).
    Other rustls defaults (key-exchange groups, signature
    schemes, ALPN preferences) are unchanged.
  - HTTPS responses now carry
    `Strict-Transport-Security: max-age=63072000` (2 years,
    matching the hstspreload.org minimum). `includeSubDomains`
    is intentionally omitted so deployments that host
    non-HTTPS services on adjacent subdomains aren't broken;
    operators that want subdomain coverage can add it at the
    upstream proxy. Per RFC 6797 §7.2 the header is only
    emitted on the HTTPS serve path, never on plain HTTP.

## [0.4.1]

- Added crate-level doc comment.
- Added doc comment to the standalone binary.

## [0.4.0]

- Added optional `https` Cargo feature: TLS support via rustls
  (vendored crypto backend, no system OpenSSL headers) with
  HTTP/1.1 + HTTP/2 ALPN negotiation. New public API:
  `TlsConfig::from_pem(cert_pem, key_pem)` for PEM-encoded
  certificates and private keys, and
  `MechanicsServer::run_tls(bind_addr, tls_config)` which
  starts the HTTPS server in a dedicated thread (mirrors
  `run()` for plain HTTP). The existing `run()` method is
  unchanged and works without the feature enabled.

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
