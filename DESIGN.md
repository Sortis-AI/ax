# Design

## Philosophy

Agent-first, human-compatible. Every decision prioritizes programmatic consumption:
- Structured output (JSON default in NO_DNA mode)
- Deterministic exit codes
- No interactive prompts when `NO_DNA=1`
- Rate-limit-aware retries

## Key decisions

### Enum-based auth instead of trait objects

`AuthProvider` is an enum, not a `dyn` trait. Avoids `async_trait` dependency and dyn-compatibility issues with async methods. Three variants: OAuth2, OAuth1, Bearer.

### Token encryption

Tokens at rest are AES-256-GCM encrypted. Key is derived from machine ID — not perfect security, but prevents casual token theft if the file is copied. The threat model assumes the machine itself is trusted.

### Output system

The `Renderable` trait provides four render methods. Each API type implements all four. This keeps rendering logic co-located with the data types rather than scattered across formatter modules.

### NO_DNA compliance

Detection: `std::env::var("NO_DNA").is_ok_and(|v| !v.is_empty())`. Affects output mode default, error formatting, interactivity, verbosity, and visual elements. CLI flags always override.

## Distribution points

- **crates.io**: `cargo install agent-x` (binary name: `ax`)
- **SKILL.md**: Agentskills spec for agent discovery

### Non-interactive OAuth split flow

Agents can't open browsers or run persistent callback servers. The flow is split into two commands:

- `ax auth login --no-browser` — generates PKCE, saves encrypted `PendingAuth` state, prints the authorization URL, exits immediately
- `ax auth callback <token>` — accepts a base64-encoded `{code, state}` blob, loads the pending auth, validates state, exchanges the code for tokens

The bridge is a static site at `https://oauth.cli.city/` (GitHub Pages) that catches X.com's redirect and encodes the callback params as a single base64 string. One copy, one paste, one command.

Design constraints:
- **Single string**: The user copies exactly one value. No multi-field paste, no URL parsing.
- **No server**: The static site is pure client-side JS. No backend, no data leaves the browser.
- **TTL**: Pending auth expires after 10 minutes. Encrypted at rest with the same AES-256-GCM scheme as tokens.
- **NO_DNA auto-activation**: `NO_DNA=1` implies `--no-browser` — agents never get the interactive flow.

## Roadmap

- [ ] Media upload support (`--media` flag)
- [ ] Per-request OAuth 1.0a signing (currently uses placeholder URL)
- [ ] Streaming API support
- [ ] Batch operations
- [ ] `ax` shell completions
- [ ] CI/CD pipeline (GitHub Actions)
