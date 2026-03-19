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
