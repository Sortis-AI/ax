# Map

## Actors

- **User/Agent** → invokes `ax` CLI
- **ax binary** → parses args, resolves auth, makes API calls, renders output
- **X API v2** (`api.x.com`) → Twitter/X backend
- **Token store** (`$XDG_DATA_HOME/agent-x/tokens.json`) → encrypted OAuth 2.0 tokens

## Data flows

```
User/Agent
  │
  ├─ ax tweet post "text"
  │    → resolve_auth() → AuthProvider
  │    → XClient.post("/tweets", body)
  │    → X API v2 (POST https://api.x.com/2/tweets)
  │    → Response → Tweet → render(mode) → stdout
  │
  ├─ ax auth login
  │    → generate PKCE challenge
  │    → bind 127.0.0.1:{port}
  │    → open browser → x.com/i/oauth2/authorize
  │    → receive callback → exchange code → token
  │    → encrypt → $XDG_DATA_HOME/agent-x/tokens.json
  │
  └─ ax auth status
       → resolve_auth() → AuthProvider.method_name()
       → load_tokens() → expiry, scopes
       → render(mode) → stdout
```

## Port map

| Port | Usage |
|------|-------|
| 8477 (default) | OAuth 2.0 PKCE callback server (configurable via `--port`) |

## File locations

| Path | Purpose |
|------|---------|
| `$XDG_DATA_HOME/agent-x/tokens.json` | Encrypted OAuth 2.0 tokens (0600) |

## Environment variables

| Variable | Purpose |
|----------|---------|
| `NO_DNA` | Enable agent-friendly mode (JSON output, no interactivity) |
| `X_CLIENT_ID` | OAuth 2.0 client ID (required for `ax auth login`) |
| `X_API_KEY` | OAuth 1.0a consumer key |
| `X_API_SECRET` | OAuth 1.0a consumer secret |
| `X_ACCESS_TOKEN` | OAuth 1.0a access token |
| `X_ACCESS_TOKEN_SECRET` | OAuth 1.0a access token secret |
| `X_BEARER_TOKEN` | Bearer token (read-only) |
| `NO_COLOR` | Disable terminal colors (respected by `colored` crate) |
