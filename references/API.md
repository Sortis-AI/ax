# X API v2 Endpoint Reference

Base URL: `https://api.x.com/2`

## Tweets

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/tweets` | POST | Create a tweet |
| `/tweets/:id` | GET | Get a tweet by ID |
| `/tweets/:id` | DELETE | Delete a tweet |
| `/tweets/search/recent` | GET | Search recent tweets (7 days) |

### Tweet fields

`id`, `text`, `author_id`, `created_at`, `public_metrics`, `conversation_id`, `in_reply_to_user_id`, `edit_history_tweet_ids`

### Public metrics

`retweet_count`, `reply_count`, `like_count`, `quote_count`, `bookmark_count`, `impression_count`

### Create tweet body

```json
{"text": "Hello world"}
```

Reply:
```json
{"text": "Reply text", "reply": {"in_reply_to_tweet_id": "123"}}
```

Quote:
```json
{"text": "Quote text", "quote_tweet_id": "123"}
```

## Users

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/users/by/username/:username` | GET | Lookup user by username |
| `/users/me` | GET | Get authenticated user |
| `/users/:id/tweets` | GET | User timeline |
| `/users/:id/followers` | GET | User's followers |
| `/users/:id/following` | GET | Users followed by user |

### User fields

`id`, `name`, `username`, `description`, `created_at`, `public_metrics`, `verified`, `profile_image_url`

## Self operations

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/users/:id/mentions` | GET | Authenticated user's mentions |
| `/users/:id/bookmarks` | GET | Authenticated user's bookmarks |
| `/users/:id/bookmarks` | POST | Bookmark a tweet |
| `/users/:id/bookmarks/:tweet_id` | DELETE | Remove bookmark |
| `/users/:id/likes` | POST | Like a tweet |
| `/users/:id/likes/:tweet_id` | DELETE | Unlike a tweet |
| `/users/:id/retweets` | POST | Retweet |
| `/users/:id/retweets/:tweet_id` | DELETE | Undo retweet |

## Pagination

List endpoints return:
```json
{
  "data": [...],
  "meta": {
    "result_count": 10,
    "next_token": "...",
    "previous_token": "..."
  }
}
```

Use `?pagination_token=<next_token>` for next page.

## Rate limits

Response headers:
- `x-rate-limit-remaining`: Requests remaining in window
- `x-rate-limit-reset`: Unix timestamp when window resets

## Authentication

### OAuth 2.0 PKCE

Authorization: `https://x.com/i/oauth2/authorize`
Token: `https://api.x.com/2/oauth2/token`

### OAuth 1.0a

HMAC-SHA1 signed `Authorization` header.

### Bearer token

`Authorization: Bearer <token>` header.

## Error responses

```json
{
  "errors": [
    {
      "message": "...",
      "type": "..."
    }
  ]
}
```

HTTP status codes: 200 (success), 401 (unauthorized), 403 (forbidden), 404 (not found), 429 (rate limited).
