---
name: agent-x
description: >
  Interact with X (Twitter) from the command line — post tweets, search, manage bookmarks,
  view timelines, and more. Use when the task involves reading or writing to X/Twitter,
  managing an X account, or automating social media workflows. Binary: ax.
  NO_DNA compliant: set NO_DNA=1 for structured JSON output.
license: GPL-3.0
compatibility: Requires Rust toolchain (cargo) or pre-built binary. Requires X API credentials (OAuth 2.0, OAuth 1.0a, or Bearer token).
metadata:
  author: chris
  version: "0.5.3"
  tags: "twitter,x,social-media,api,cli"
---

## Installation

### Install Rust toolchain (if not present)

Check if `cargo` is available:

```bash
cargo --version
```

If not installed, install via rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

### Install ax

```bash
cargo install agent-x
```

The binary is named `ax`. Verify:

```bash
ax --version
```

### Update ax

```bash
cargo install agent-x --force
```

This rebuilds and replaces the existing binary with the latest version from crates.io.

## Authentication

Three methods, resolved in priority order:

### 1. OAuth 2.0 PKCE (recommended)

**Interactive** (opens browser, local callback server):

```bash
ax auth login
```

**Non-interactive** (for agents — no browser, no server):

```bash
ax auth login --no-browser
```

Prints an authorization URL. The user opens it, authorizes on x.com, and is redirected to `https://oauth.cli.city/` which displays a base64-encoded token. Complete the flow:

```bash
ax auth callback <base64-token>
```

In NO_DNA mode, `ax auth login` automatically uses the non-interactive flow and outputs JSON:

```json
{"action_required":"open_url","url":"https://x.com/i/oauth2/authorize?..."}
```

Tokens are encrypted at `$XDG_DATA_HOME/agent-x/tokens.json` with 2h expiry and automatic refresh.

#### Do not run `ax auth login` when already authenticated

`ax auth login` will fail with an error if stored tokens already exist. This is intentional — running login twice without logging out first causes a state mismatch that makes the second callback fail silently. If you receive this error, **stop**. You are already authenticated. Verify with `ax auth status`. Only proceed with a new login flow after running `ax auth logout`.

#### Callback failure is a hard stop

**If `ax auth callback` fails for any reason, stop immediately.** Do not attempt to:
- Decode the base64 token yourself
- Re-encode it in a different format
- Use `--code` and `--state` flags as an alternative

None of these will work. The token is correct or it isn't — the failure is in the auth state, not the encoding. Trying to manipulate the token wastes effort and cannot succeed.

**What to do instead:** Tell the user the auth failed, show them the exact error message, and ask them to restart the flow:

```bash
ax auth login --no-browser
```

Then repeat the process from the top.

#### Common failure reasons

- **Expired** — Auth state expires after 10 minutes from when `ax auth login --no-browser` ran. If the user was slow to authorize or paste the token, restart the flow.
- **Stale state** — If `ax auth login --no-browser` was run more than once before `ax auth callback`, only the most recent token is valid. Show the user the error and restart.
- **Token exchange failure** — If the callback fails with `"Token exchange failed: ..."`, the authorization code from X was rejected. Codes are single-use and expire within seconds. Do not retry `ax auth callback` with the same token — the code is dead. Restart from `ax auth login --no-browser`.
- **Whitespace in token** — Pass the token as a single unbroken string with no spaces, newlines, or markdown formatting characters.

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

Check status: `ax auth status` | Log out: `ax auth logout`

## Command overview

```
ax [--output json|plain|markdown|human] [--verbose]
├── tweet post [--community-id ID]|get|delete|reply|quote|search|metrics
├── user get|timeline|followers|following
├── self mentions|bookmarks|like|unlike|retweet|unretweet|bookmark|unbookmark
├── community search|get|post
└── auth login [--no-browser]|callback|status|logout
```

See [references/commands.md](references/commands.md) for full command reference with examples.

## NO_DNA mode

Set `NO_DNA=1` for agent-friendly behavior:

```bash
export NO_DNA=1
ax tweet get 123456   # stdout: JSON, no colors, no spinners
```

- **Output**: JSON to stdout by default
- **Errors**: JSON to stderr with `error`, `error_type`, `timestamp` fields
- **Interactivity**: None (auth URLs printed as JSON, no browser launch)
- **Visual**: No colors, no progress bars

CLI flags (`--output`) always override NO_DNA defaults.

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

- **Rate limits**: X API v2 has per-endpoint rate limits. `ax` tracks and waits automatically, but heavy usage may still hit limits.
- **Token refresh**: OAuth 2.0 refresh tokens are one-time-use. If a refresh fails, run `ax auth login` again.
- **Media upload**: Not yet implemented. The `--media` flag is accepted but currently ignored.

## Reference

- [references/commands.md](references/commands.md) — Full command reference with examples
- [references/API.md](references/API.md) — X API v2 endpoint documentation
