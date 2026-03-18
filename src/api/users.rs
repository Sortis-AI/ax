use crate::api::types::{ApiResponse, Tweet, TweetList, User, UserList};
use crate::api::XClient;
use crate::error::AgentXError;

const USER_FIELDS: &str =
    "id,name,username,description,created_at,public_metrics,verified,profile_image_url";
const TWEET_FIELDS: &str = "id,text,author_id,created_at,public_metrics,conversation_id";

impl XClient {
    pub async fn get_user(&self, username: &str) -> Result<User, AgentXError> {
        let query = vec![("user.fields".to_string(), USER_FIELDS.to_string())];
        let resp = self
            .get(&format!("/users/by/username/{username}"), &query)
            .await?;
        let api: ApiResponse<User> = resp.json().await?;
        api.data
            .ok_or_else(|| AgentXError::NotFound(format!("User @{username} not found")))
    }

    /// Resolve a username to a user ID. If the input is already numeric, return it as-is.
    async fn resolve_user_id(&self, user: &str) -> Result<String, AgentXError> {
        if user.chars().all(|c| c.is_ascii_digit()) {
            Ok(user.to_string())
        } else {
            let u = self.get_user(user).await?;
            Ok(u.id)
        }
    }

    pub async fn get_user_timeline(
        &self,
        user: &str,
        max_results: u32,
        next_token: &Option<String>,
    ) -> Result<TweetList, AgentXError> {
        let user_id = self.resolve_user_id(user).await?;
        let mut query = vec![("tweet.fields".to_string(), TWEET_FIELDS.to_string())];
        crate::api::pagination::apply_pagination_params(&mut query, max_results, next_token);

        let resp = self
            .get(&format!("/users/{user_id}/tweets"), &query)
            .await?;
        let api: ApiResponse<Vec<Tweet>> = resp.json().await?;

        Ok(TweetList {
            tweets: api.data.unwrap_or_default(),
            next_token: api.meta.as_ref().and_then(|m| m.next_token.clone()),
            result_count: api.meta.as_ref().and_then(|m| m.result_count),
        })
    }

    pub async fn get_user_followers(
        &self,
        user: &str,
        max_results: u32,
        next_token: &Option<String>,
    ) -> Result<UserList, AgentXError> {
        let user_id = self.resolve_user_id(user).await?;
        let mut query = vec![("user.fields".to_string(), USER_FIELDS.to_string())];
        crate::api::pagination::apply_pagination_params(&mut query, max_results, next_token);

        let resp = self
            .get(&format!("/users/{user_id}/followers"), &query)
            .await?;
        let api: ApiResponse<Vec<User>> = resp.json().await?;

        Ok(UserList {
            users: api.data.unwrap_or_default(),
            next_token: api.meta.as_ref().and_then(|m| m.next_token.clone()),
            result_count: api.meta.as_ref().and_then(|m| m.result_count),
        })
    }

    pub async fn get_user_following(
        &self,
        user: &str,
        max_results: u32,
        next_token: &Option<String>,
    ) -> Result<UserList, AgentXError> {
        let user_id = self.resolve_user_id(user).await?;
        let mut query = vec![("user.fields".to_string(), USER_FIELDS.to_string())];
        crate::api::pagination::apply_pagination_params(&mut query, max_results, next_token);

        let resp = self
            .get(&format!("/users/{user_id}/following"), &query)
            .await?;
        let api: ApiResponse<Vec<User>> = resp.json().await?;

        Ok(UserList {
            users: api.data.unwrap_or_default(),
            next_token: api.meta.as_ref().and_then(|m| m.next_token.clone()),
            result_count: api.meta.as_ref().and_then(|m| m.result_count),
        })
    }
}
