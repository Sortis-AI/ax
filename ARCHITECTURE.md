# Architecture

## Overview

`agent-x` is a Rust CLI that wraps the X (Twitter) API v2. It's designed agent-first: structured output by default in NO_DNA mode, deterministic exit codes, no interactive prompts when consumed programmatically.

## Components

### CLI Layer (`src/cli/`)

Clap-derive structs defining the command tree. Each domain has its own subcommand file:
- `mod.rs` — `Cli` struct with global flags, `Command` enum
- `tweet.rs` — `TweetAction` (post, get, delete, reply, quote, search, metrics)
- `user.rs` — `UserAction` (get, timeline, followers, following)
- `self_ops.rs` — `SelfAction` (mentions, bookmarks, like/unlike, retweet/unretweet, bookmark/unbookmark)
- `auth.rs` — `AuthAction` (login, status, logout)

### API Layer (`src/api/`)

- `mod.rs` — `XClient` struct with HTTP client, auth, rate limiting, retry logic
- `types.rs` — Serde types for X API v2 responses (`Tweet`, `User`, `ApiResponse<T>`, etc.) with `Renderable` impls
- `tweets.rs` — Tweet CRUD + search + metrics
- `users.rs` — User lookup + timeline + followers/following
- `self_ops.rs` — Authenticated user operations (mentions, bookmarks, likes, retweets)
- `pagination.rs` — Pagination query param helper

### Auth Layer (`src/auth/`)

- `mod.rs` — `AuthProvider` enum (OAuth2, OAuth1, Bearer), `resolve_auth()` resolution
- `oauth1.rs` — HMAC-SHA1 signature generation for OAuth 1.0a
- `oauth2.rs` — PKCE flow (challenge gen, callback server, token exchange, refresh)
- `token_store.rs` — AES-256-GCM encrypted token storage at XDG_DATA_HOME
- `refresh.rs` — Placeholder for future refresh scheduling

### Output Layer (`src/output/`)

- `mod.rs` — `OutputMode` enum, `Renderable` trait, `print_output()` helper
- `json.rs`, `plain.rs`, `markdown.rs`, `human.rs` — Per-mode rendering (logic lives in `Renderable` impls on types)

### Config (`src/config.rs`)

`RuntimeConfig` — merges CLI flags with environment (NO_DNA detection).

### Error (`src/error.rs`)

`AgentXError` enum via thiserror. Maps to exit codes. NO_DNA-aware stderr reporting (JSON errors).

## Data flow

```
CLI args → Cli::parse() → RuntimeConfig
         → resolve_auth() → AuthProvider
         → XClient::new(auth)
         → handle_{tweet,user,self,auth}()
         → XClient.{get,post,delete}()  ← rate limit + retry
         → Response → serde deserialize → Renderable type
         → print_output(item, mode) → stdout
```

## Auth resolution order

1. Stored OAuth 2.0 tokens (`$XDG_DATA_HOME/agent-x/tokens.json`)
2. OAuth 1.0a env vars (`X_API_KEY`, `X_API_SECRET`, `X_ACCESS_TOKEN`, `X_ACCESS_TOKEN_SECRET`)
3. Bearer token env var (`X_BEARER_TOKEN`)

## Rate limiting

`XClient` tracks `x-rate-limit-remaining` and `x-rate-limit-reset` headers per endpoint. Preemptively waits when remaining hits 0. Retries up to 3 times on 429 and once on 401 (with token refresh).

## Token storage

Tokens encrypted with AES-256-GCM. Key derived from `/etc/machine-id` (Linux) via SHA-256. File permissions set to 0600.
