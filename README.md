# agent-x

Agent-first Twitter/X CLI. Binary: **`ax`**. Published as `agent-x` on crates.io.

Agents are the primary consumer, humans secondary. Full [NO_DNA](https://no-dna.org) compliance for structured, machine-readable output.

## Install

```bash
cargo install agent-x
```

Requires Rust 1.75+.

## Quick start

```bash
# Authenticate (OAuth 2.0 PKCE)
export X_CLIENT_ID="your-client-id"
ax auth login

# Post a tweet
ax tweet post "Hello from agent-x!"

# Get a tweet
ax tweet get 1234567890

# Search
ax tweet search "rust lang" --max-results 20

# User lookup
ax user get elonmusk
```

## Authentication

Three methods, resolved in priority order:

1. **OAuth 2.0 PKCE** — `ax auth login` (recommended, stored encrypted)
   - Interactive: opens browser, local callback server
   - Non-interactive: `ax auth login --no-browser` → prints URL → `ax auth callback <token>`
2. **OAuth 1.0a** — env vars `X_API_KEY`, `X_API_SECRET`, `X_ACCESS_TOKEN`, `X_ACCESS_TOKEN_SECRET`
3. **Bearer token** — env var `X_BEARER_TOKEN` (read-only)

## Command tree

```
ax [--output json|plain|markdown|human] [--verbose]
├── tweet post|get|delete|reply|quote|search|metrics
├── user get|timeline|followers|following
├── self mentions|bookmarks|like|unlike|retweet|unretweet|bookmark|unbookmark
└── auth login [--no-browser]|callback|status|logout
```

See `ax --help`, `ax <command> --help`, or [SKILL.md](SKILL.md) for full usage.

## Output modes

| Mode | Flag | Description |
|------|------|-------------|
| `json` | `-o json` | JSON (default when `NO_DNA=1`) |
| `plain` | `-o plain` | TSV for piping |
| `markdown` | `-o markdown` | Markdown tables |
| `human` | `-o human` | Rich terminal (default) |

## NO_DNA mode

```bash
NO_DNA=1 ax tweet get 123456
```

- JSON stdout, JSON stderr errors, no colors, no interactivity, ISO 8601 timestamps.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Auth error |
| 3 | Not found |
| 4 | Rate limited |
| 5 | API error |

## Development

```bash
cargo build           # Build
cargo test            # Run tests
cargo clippy          # Lint
cargo run -- --help   # Run locally
```

## Project layout

```
agent-x/
├── Cargo.toml          # Package manifest
├── SKILL.md            # Agentskills spec
├── references/API.md   # X API v2 endpoint reference
├── src/
│   ├── main.rs         # Entry point, CLI dispatch
│   ├── config.rs       # RuntimeConfig (NO_DNA, output, verbosity)
│   ├── error.rs        # AgentXError enum, exit codes
│   ├── cli/            # Clap command definitions
│   ├── api/            # XClient, API types, endpoint impls
│   ├── auth/           # OAuth 2.0 PKCE, OAuth 1.0a, Bearer, token storage
│   └── output/         # Renderable trait, JSON/plain/markdown/human renderers
└── tests/              # Integration tests + fixtures
```

## License

GPL-3.0
