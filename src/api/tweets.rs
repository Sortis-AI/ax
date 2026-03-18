use crate::api::types::{ApiResponse, MutationResult, Tweet, TweetList};
use crate::api::XClient;
use crate::error::AgentXError;

const TWEET_FIELDS: &str = "id,text,author_id,created_at,public_metrics,conversation_id,in_reply_to_user_id,edit_history_tweet_ids";

impl XClient {
    pub async fn get_tweet(
        &self,
        id: &str,
        fields: Option<&str>,
        expansions: Option<&str>,
    ) -> Result<Tweet, AgentXError> {
        let mut query = vec![(
            "tweet.fields".to_string(),
            fields.unwrap_or(TWEET_FIELDS).to_string(),
        )];
        if let Some(exp) = expansions {
            query.push(("expansions".to_string(), exp.to_string()));
        }

        let resp = self.get(&format!("/tweets/{id}"), &query).await?;
        let api: ApiResponse<Tweet> = resp.json().await?;

        api.data
            .ok_or_else(|| AgentXError::NotFound(format!("Tweet {id} not found")))
    }

    pub async fn post_tweet(&self, text: &str) -> Result<Tweet, AgentXError> {
        let body = serde_json::json!({ "text": text });
        let resp = self.post("/tweets", body).await?;
        let api: ApiResponse<Tweet> = resp.json().await?;
        api.data.ok_or_else(|| AgentXError::Api {
            status: 0,
            message: "No data in tweet post response".to_string(),
        })
    }

    pub async fn delete_tweet(&self, id: &str) -> Result<MutationResult, AgentXError> {
        let resp = self.delete(&format!("/tweets/{id}")).await?;
        let body: serde_json::Value = resp.json().await?;
        let deleted = body
            .get("data")
            .and_then(|d| d.get("deleted"))
            .and_then(|d| d.as_bool())
            .unwrap_or(false);
        Ok(MutationResult {
            action: "delete_tweet".to_string(),
            success: deleted,
            id: Some(id.to_string()),
        })
    }

    pub async fn reply_tweet(&self, id: &str, text: &str) -> Result<Tweet, AgentXError> {
        let body = serde_json::json!({
            "text": text,
            "reply": { "in_reply_to_tweet_id": id }
        });
        let resp = self.post("/tweets", body).await?;
        let api: ApiResponse<Tweet> = resp.json().await?;
        api.data.ok_or_else(|| AgentXError::Api {
            status: 0,
            message: "No data in reply response".to_string(),
        })
    }

    pub async fn quote_tweet(&self, id: &str, text: &str) -> Result<Tweet, AgentXError> {
        let body = serde_json::json!({
            "text": text,
            "quote_tweet_id": id
        });
        let resp = self.post("/tweets", body).await?;
        let api: ApiResponse<Tweet> = resp.json().await?;
        api.data.ok_or_else(|| AgentXError::Api {
            status: 0,
            message: "No data in quote response".to_string(),
        })
    }

    pub async fn search_tweets(
        &self,
        query_str: &str,
        max_results: u32,
        next_token: &Option<String>,
    ) -> Result<TweetList, AgentXError> {
        let mut query = vec![
            ("query".to_string(), query_str.to_string()),
            ("tweet.fields".to_string(), TWEET_FIELDS.to_string()),
        ];
        crate::api::pagination::apply_pagination_params(&mut query, max_results, next_token);

        let resp = self.get("/tweets/search/recent", &query).await?;
        let api: ApiResponse<Vec<Tweet>> = resp.json().await?;

        Ok(TweetList {
            tweets: api.data.unwrap_or_default(),
            next_token: api.meta.as_ref().and_then(|m| m.next_token.clone()),
            result_count: api.meta.as_ref().and_then(|m| m.result_count),
        })
    }

    pub async fn get_tweet_metrics(&self, id: &str) -> Result<Tweet, AgentXError> {
        self.get_tweet(id, Some("id,text,public_metrics"), None)
            .await
    }
}
