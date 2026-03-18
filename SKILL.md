---
name: agent-x
description: >
  Interact with X (Twitter) from the command line — post tweets, search, manage bookmarks,
  view timelines, and more. Use when the task involves reading or writing to X/Twitter,
  managing an X account, or automating social media workflows. Binary: ax.
  NO_DNA compliant: set NO_DNA=1 for structured JSON output.
license: MIT
compatibility: Requires X API credentials (OAuth 2.0 or OAuth 1.0a). Binary: ax. Install via cargo install agent-x.
metadata:
  author: chris
  version: "0.1.0"
  tags: "twitter,x,social-media,api,cli"
---

## When to use this skill

- Posting, replying to, or quoting tweets
- Searching tweets by keyword or hashtag
- Reading a user's timeline, followers, or following list
- Managing bookmarks, likes, and retweets on your account
- Checking tweet engagement metrics
- Automating X/Twitter workflows in scripts or agent pipelines

## Quick start

```bash
cargo install agent-x
ax auth login          # OAuth 2.0 PKCE flow (opens browser)
ax tweet post "hello"  # Post a tweet
ax tweet get 123456    # Get a tweet by ID
```

## Authentication

Three methods, resolved in priority order:

### 1. OAuth 2.0 PKCE (recommended)

```bash
export X_CLIENT_ID="your-client-id"
ax auth login
```

Opens a browser for authorization. Tokens are encrypted and stored at `$XDG_DATA_HOME/agent-x/tokens.json`. Tokens auto-refresh (2h expiry, one-time-use refresh tokens).

In NO_DNA mode, `ax auth login` outputs JSON with the authorization URL instead of opening a browser:
```json
{"action_required":"open_url","url":"https://x.com/i/oauth2/authorize?..."}
```

### 2. OAuth 1.0a (env vars)

```bash
export X_API_KEY="..."
export X_API_SECRET="..."
export X_ACCESS_TOKEN="..."
export X_ACCESS_TOKEN_SECRET="..."
```

### 3. Bearer token (read-only)

```bash
export X_BEARER_TOKEN="..."
```

Check status: `ax auth status`
Log out: `ax auth logout`

## Commands

### Tweet operations

```bash
ax tweet post "Hello world"                  # Post a tweet
ax tweet post "With image" --media photo.jpg # Post with media (planned)
ax tweet get <id>                            # Get tweet by ID
ax tweet get <id> --fields id,text           # Get specific fields
ax tweet delete <id>                         # Delete a tweet
ax tweet reply <id> "Nice post!"             # Reply to a tweet
ax tweet quote <id> "This is great"          # Quote tweet
ax tweet search "rust lang" --max-results 20 # Search recent tweets
ax tweet metrics <id>                        # Get engagement metrics
```

### User operations

```bash
ax user get elonmusk                             # Get user profile
ax user timeline elonmusk --max-results 5         # User's recent tweets
ax user followers elonmusk --max-results 100      # User's followers
ax user following elonmusk --max-results 100      # Who user follows
```

### Self operations (authenticated user)

```bash
ax self mentions --max-results 20     # Your recent mentions
ax self bookmarks                     # Your bookmarks
ax self like <id>                     # Like a tweet
ax self unlike <id>                   # Unlike a tweet
ax self retweet <id>                  # Retweet
ax self unretweet <id>                # Undo retweet
ax self bookmark <id>                # Bookmark a tweet
ax self unbookmark <id>              # Remove bookmark
```

### Auth operations

```bash
ax auth login                         # OAuth 2.0 PKCE login
ax auth login --scopes "tweet.read"   # Login with specific scopes
ax auth login --port 9090             # Use custom callback port
ax auth status                        # Show auth status
ax auth logout                        # Remove stored tokens
```

### Global flags

```bash
ax --output json tweet get <id>       # Force JSON output
ax -o plain tweet search "query"      # TSV output for piping
ax -o markdown user get someone       # Markdown output
ax -o human tweet get <id>            # Rich terminal output
ax --verbose tweet get <id>           # Verbose mode
```

## NO_DNA mode

Set `NO_DNA=1` for agent-friendly behavior:

```bash
export NO_DNA=1
ax tweet get 123456   # stdout: JSON, no colors, no spinners
```

When NO_DNA is set:
- **Output**: JSON to stdout by default
- **Errors**: JSON to stderr with `error`, `error_type`, `timestamp` fields
- **Timestamps**: ISO 8601 (no relative times)
- **Interactivity**: None (auth URLs printed as JSON, no browser launch)
- **Verbosity**: Forced on
- **Visual**: No colors, no progress bars

CLI flags (`--output`) always override NO_DNA defaults.

## Pagination

Commands that return lists support `--max-results` and `--next-token`:

```bash
ax tweet search "rust" --max-results 10
# Response includes next_token if more results exist
ax tweet search "rust" --max-results 10 --next-token "abc123"
```

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Authentication error |
| 3 | Not found |
| 4 | Rate limited |
| 5 | API error |

## Gotchas

- **Rate limits**: X API v2 has per-endpoint rate limits. `ax` tracks them and waits automatically, but heavy usage may still hit limits.
- **Reply restrictions**: Some tweets restrict who can reply. Attempting to reply may fail with an API error.
- **Token refresh**: OAuth 2.0 refresh tokens are one-time-use. If a refresh fails, you need to `ax auth login` again.
- **Media upload**: Not yet implemented. The `--media` flag is accepted but currently ignored.
- **OAuth 1.0a signing**: Currently signs against a placeholder URL. Full per-request signing is planned.

## Reference

See [references/API.md](references/API.md) for full X API v2 endpoint documentation.
