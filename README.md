# mechanics-rs
HTTP server wrapper around [mechanics-core](https://crates.io/crates/mechanics-core).

## Endpoint

- `POST /api/v1/mechanics`
- `Content-Type: application/json`
- `Authorization: Bearer <token>`

The server validates Bearer tokens configured through `MECHANICS_ALLOWED_TOKENS`
(comma-separated). If no tokens are configured, the server intentionally runs in
deny-all mode and returns `401 Unauthorized` for every request until tokens are
added.

## Quick start

```bash
LISTEN_ADDR=127.0.0.1:3001 \
MECHANICS_ALLOWED_TOKENS=token-a,token-b \
cargo run --bin mechanics-rs
```

## Example request

```bash
curl -X POST http://127.0.0.1:3001/api/v1/mechanics \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer token-a' \
  -d '{}'
```

## License

**This crate is dual-licensed under `Apache-2.0 OR MPL-2.0`**;
either license is sufficient; choose whichever fits your project.

**Rationale**: We generally want our reusable Rust crates to be
under a license permissive enough to be friendly for the Rust
community as a whole, while maintaining GPL-2.0 compatibility via
the MPL-2.0 arm. This is FSF-safer for everyone than `MIT OR Apache-2.0`,
still being permissive. **This is the standard licensing** for our reusable
Rust crate projects. Someone's `GPL-2.0-or-later` project should not be
forced to drop the `GPL-2.0` option because of our crates,
while `Apache-2.0` is the non-copyleft (permissive) license recommended
by the FSF, which we base our decisions on.

