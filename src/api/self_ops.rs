use crate::api::types::{ApiResponse, MutationResult, Tweet, TweetList};
use crate::api::XClient;
use crate::error::AgentXError;

const TWEET_FIELDS: &str = "id,text,author_id,created_at,public_metrics,conversation_id";

impl XClient {
    /// Get the authenticated user's ID from /users/me.
    async fn get_me_id(&self) -> Result<String, AgentXError> {
        let resp = self.get("/users/me", &[]).await?;
        let api: ApiResponse<serde_json::Value> = resp.json().await?;
        let data = api
            .data
            .ok_or_else(|| AgentXError::Auth("Cannot determine authenticated user".to_string()))?;
        data.get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AgentXError::Auth("No user ID in /users/me response".to_string()))
    }

    pub async fn get_mentions(
        &self,
        max_results: u32,
        next_token: &Option<String>,
    ) -> Result<TweetList, AgentXError> {
        let me = self.get_me_id().await?;
        let mut query = vec![("tweet.fields".to_string(), TWEET_FIELDS.to_string())];
        crate::api::pagination::apply_pagination_params(&mut query, max_results, next_token);

        let resp = self.get(&format!("/users/{me}/mentions"), &query).await?;
        let api: ApiResponse<Vec<Tweet>> = resp.json().await?;

        Ok(TweetList {
            tweets: api.data.unwrap_or_default(),
            next_token: api.meta.as_ref().and_then(|m| m.next_token.clone()),
            result_count: api.meta.as_ref().and_then(|m| m.result_count),
        })
    }

    pub async fn get_bookmarks(
        &self,
        max_results: u32,
        next_token: &Option<String>,
    ) -> Result<TweetList, AgentXError> {
        let me = self.get_me_id().await?;
        let mut query = vec![("tweet.fields".to_string(), TWEET_FIELDS.to_string())];
        crate::api::pagination::apply_pagination_params(&mut query, max_results, next_token);

        let resp = self.get(&format!("/users/{me}/bookmarks"), &query).await?;
        let api: ApiResponse<Vec<Tweet>> = resp.json().await?;

        Ok(TweetList {
            tweets: api.data.unwrap_or_default(),
            next_token: api.meta.as_ref().and_then(|m| m.next_token.clone()),
            result_count: api.meta.as_ref().and_then(|m| m.result_count),
        })
    }

    pub async fn like_tweet(&self, tweet_id: &str) -> Result<MutationResult, AgentXError> {
        let me = self.get_me_id().await?;
        let body = serde_json::json!({ "tweet_id": tweet_id });
        let resp = self.post(&format!("/users/{me}/likes"), body).await?;
        let val: serde_json::Value = resp.json().await?;
        let liked = val
            .get("data")
            .and_then(|d| d.get("liked"))
            .and_then(|l| l.as_bool())
            .unwrap_or(false);
        Ok(MutationResult {
            action: "like".to_string(),
            success: liked,
            id: Some(tweet_id.to_string()),
        })
    }

    pub async fn unlike_tweet(&self, tweet_id: &str) -> Result<MutationResult, AgentXError> {
        let me = self.get_me_id().await?;
        let resp = self
            .delete(&format!("/users/{me}/likes/{tweet_id}"))
            .await?;
        let val: serde_json::Value = resp.json().await?;
        let unliked = val
            .get("data")
            .and_then(|d| d.get("liked"))
            .and_then(|l| l.as_bool())
            .map(|l| !l)
            .unwrap_or(true);
        Ok(MutationResult {
            action: "unlike".to_string(),
            success: unliked,
            id: Some(tweet_id.to_string()),
        })
    }

    pub async fn retweet(&self, tweet_id: &str) -> Result<MutationResult, AgentXError> {
        let me = self.get_me_id().await?;
        let body = serde_json::json!({ "tweet_id": tweet_id });
        let resp = self.post(&format!("/users/{me}/retweets"), body).await?;
        let val: serde_json::Value = resp.json().await?;
        let retweeted = val
            .get("data")
            .and_then(|d| d.get("retweeted"))
            .and_then(|r| r.as_bool())
            .unwrap_or(false);
        Ok(MutationResult {
            action: "retweet".to_string(),
            success: retweeted,
            id: Some(tweet_id.to_string()),
        })
    }

    pub async fn unretweet(&self, tweet_id: &str) -> Result<MutationResult, AgentXError> {
        let me = self.get_me_id().await?;
        let resp = self
            .delete(&format!("/users/{me}/retweets/{tweet_id}"))
            .await?;
        let val: serde_json::Value = resp.json().await?;
        let unretweeted = val
            .get("data")
            .and_then(|d| d.get("retweeted"))
            .and_then(|r| r.as_bool())
            .map(|r| !r)
            .unwrap_or(true);
        Ok(MutationResult {
            action: "unretweet".to_string(),
            success: unretweeted,
            id: Some(tweet_id.to_string()),
        })
    }

    pub async fn bookmark_tweet(&self, tweet_id: &str) -> Result<MutationResult, AgentXError> {
        let me = self.get_me_id().await?;
        let body = serde_json::json!({ "tweet_id": tweet_id });
        let resp = self.post(&format!("/users/{me}/bookmarks"), body).await?;
        let val: serde_json::Value = resp.json().await?;
        let bookmarked = val
            .get("data")
            .and_then(|d| d.get("bookmarked"))
            .and_then(|b| b.as_bool())
            .unwrap_or(false);
        Ok(MutationResult {
            action: "bookmark".to_string(),
            success: bookmarked,
            id: Some(tweet_id.to_string()),
        })
    }

    pub async fn unbookmark_tweet(&self, tweet_id: &str) -> Result<MutationResult, AgentXError> {
        let me = self.get_me_id().await?;
        let resp = self
            .delete(&format!("/users/{me}/bookmarks/{tweet_id}"))
            .await?;
        let val: serde_json::Value = resp.json().await?;
        let unbookmarked = val
            .get("data")
            .and_then(|d| d.get("bookmarked"))
            .and_then(|b| b.as_bool())
            .map(|b| !b)
            .unwrap_or(true);
        Ok(MutationResult {
            action: "unbookmark".to_string(),
            success: unbookmarked,
            id: Some(tweet_id.to_string()),
        })
    }
}
