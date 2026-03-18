# Notes

## Build discoveries

- **reqwest 0.13**: Feature renamed from `rustls-tls` to `rustls`. Also needs `query` and `form` features explicitly enabled (not bundled in default).
- **Async trait dyn-compatibility**: Rust 1.75+ supports `async fn` in traits natively, but they're not dyn-compatible. Used enum-based dispatch (`AuthProvider` enum) instead of `Box<dyn AuthProvider>` to avoid the `async_trait` crate dependency.
- **RustCrypto crate versions**: `hmac`, `sha1`, `sha2`, `aes-gcm` all have RC releases as "latest" on crates.io. Stable versions: hmac 0.12.1, sha1 0.10.6, sha2 0.10.9, aes-gcm 0.10.3.
- **`hostname` crate**: Not needed — using `/etc/machine-id` with a static fallback for token encryption key derivation.

## Known limitations

- **OAuth 1.0a signing**: Currently signs against a placeholder URL (`/users/me`). Full per-request signing requires passing method+URL context through the auth layer. Works for basic usage but will produce incorrect signatures for endpoints other than `/users/me`.
- **Media upload**: `--media` flag is accepted but not implemented. X API v2 media upload requires a multi-step process (init → append → finalize).
- **Token refresh race**: If multiple `ax` processes run concurrently and both try to refresh the same one-time-use refresh token, the second will fail. Mitigated by the 30s refresh buffer but not fully solved.
