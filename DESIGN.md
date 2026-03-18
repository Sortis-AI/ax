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

## Roadmap

- [ ] Media upload support (`--media` flag)
- [ ] Per-request OAuth 1.0a signing (currently uses placeholder URL)
- [ ] Streaming API support
- [ ] Batch operations
- [ ] `ax` shell completions
- [ ] CI/CD pipeline (GitHub Actions)
