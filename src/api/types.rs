use serde::{Deserialize, Serialize};

use crate::output::{OutputMode, Renderable};

/// Top-level X API v2 response wrapper.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub includes: Option<serde_json::Value>,
    #[serde(default)]
    pub meta: Option<ResponseMeta>,
    #[serde(default)]
    pub errors: Option<Vec<ApiError>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMeta {
    #[serde(default)]
    pub result_count: Option<u32>,
    #[serde(default)]
    pub next_token: Option<String>,
    #[serde(default)]
    pub previous_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub author_id: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub public_metrics: Option<TweetMetrics>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub in_reply_to_user_id: Option<String>,
    #[serde(default)]
    pub edit_history_tweet_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TweetMetrics {
    #[serde(default)]
    pub retweet_count: u64,
    #[serde(default)]
    pub reply_count: u64,
    #[serde(default)]
    pub like_count: u64,
    #[serde(default)]
    pub quote_count: u64,
    #[serde(default)]
    pub bookmark_count: u64,
    #[serde(default)]
    pub impression_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub username: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub public_metrics: Option<UserMetrics>,
    #[serde(default)]
    pub verified: Option<bool>,
    #[serde(default)]
    pub profile_image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserMetrics {
    #[serde(default)]
    pub followers_count: u64,
    #[serde(default)]
    pub following_count: u64,
    #[serde(default)]
    pub tweet_count: u64,
    #[serde(default)]
    pub listed_count: u64,
}

/// Wrapper for a list of tweets (search results, timelines, etc.)
#[derive(Debug, Serialize, Deserialize)]
pub struct TweetList {
    pub tweets: Vec<Tweet>,
    pub next_token: Option<String>,
    pub result_count: Option<u32>,
}

/// Wrapper for a list of users (followers, following, etc.)
#[derive(Debug, Serialize, Deserialize)]
pub struct UserList {
    pub users: Vec<User>,
    pub next_token: Option<String>,
    pub result_count: Option<u32>,
}

/// Simple success/failure response for mutations (delete, like, etc.)
#[derive(Debug, Serialize, Deserialize)]
pub struct MutationResult {
    pub action: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Auth status info
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatus {
    pub method: String,
    pub authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// OAuth 2.0 login action (for NO_DNA mode)
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AuthLoginAction {
    pub action_required: String,
    pub url: String,
}

// --- Renderable impls ---

impl Renderable for Tweet {
    fn render_human(&self) -> String {
        use colored::Colorize;
        let mut out = String::new();
        if let Some(author) = &self.author_id {
            out.push_str(&format!("{} ", format!("@{author}").cyan()));
        }
        if let Some(ts) = &self.created_at {
            out.push_str(&format!("{}\n", ts.dimmed()));
        }
        out.push_str(&self.text);
        if let Some(m) = &self.public_metrics {
            out.push_str(&format!(
                "\n{} {} {} {} {} {}",
                "♻".dimmed(),
                m.retweet_count.to_string().bold(),
                "♥".red(),
                m.like_count.to_string().bold(),
                "💬".dimmed(),
                m.reply_count.to_string().bold(),
            ));
        }
        out.push_str(&format!("\n{}", format!("ID: {}", self.id).dimmed()));
        out
    }

    fn render_plain(&self) -> String {
        let author = self.author_id.as_deref().unwrap_or("");
        let created = self.created_at.as_deref().unwrap_or("");
        let text = self.text.replace(['\t', '\n'], " ");
        format!("{}\t{}\t{}\t{}", self.id, author, created, text)
    }

    fn render_markdown(&self) -> String {
        let mut out = String::new();
        if let Some(author) = &self.author_id {
            out.push_str(&format!("**@{author}**"));
        }
        if let Some(ts) = &self.created_at {
            out.push_str(&format!(" — {ts}"));
        }
        out.push_str(&format!("\n\n{}\n", self.text));
        if let Some(m) = &self.public_metrics {
            out.push_str(&format!(
                "\n| RT | Likes | Replies | Quotes |\n|---|---|---|---|\n| {} | {} | {} | {} |\n",
                m.retweet_count, m.like_count, m.reply_count, m.quote_count
            ));
        }
        out.push_str(&format!("\n*ID: {}*", self.id));
        out
    }
}

impl Renderable for User {
    fn render_human(&self) -> String {
        use colored::Colorize;
        let mut out = format!(
            "{} ({})\n",
            self.name.bold(),
            format!("@{}", self.username).cyan()
        );
        if let Some(desc) = &self.description {
            out.push_str(&format!("{desc}\n"));
        }
        if let Some(m) = &self.public_metrics {
            out.push_str(&format!(
                "Followers: {}  Following: {}  Tweets: {}",
                m.followers_count.to_string().bold(),
                m.following_count.to_string().bold(),
                m.tweet_count.to_string().bold(),
            ));
        }
        out.push_str(&format!("\n{}", format!("ID: {}", self.id).dimmed()));
        out
    }

    fn render_plain(&self) -> String {
        let desc = self
            .description
            .as_deref()
            .unwrap_or("")
            .replace(['\t', '\n'], " ");
        format!("{}\t{}\t{}\t{}", self.id, self.username, self.name, desc)
    }

    fn render_markdown(&self) -> String {
        let mut out = format!("## {} (@{})\n", self.name, self.username);
        if let Some(desc) = &self.description {
            out.push_str(&format!("\n{desc}\n"));
        }
        if let Some(m) = &self.public_metrics {
            out.push_str(&format!(
                "\n| Followers | Following | Tweets |\n|---|---|---|\n| {} | {} | {} |\n",
                m.followers_count, m.following_count, m.tweet_count
            ));
        }
        out.push_str(&format!("\n*ID: {}*", self.id));
        out
    }
}

impl Renderable for TweetList {
    fn render_human(&self) -> String {
        use colored::Colorize;
        let mut out = String::new();
        for (i, t) in self.tweets.iter().enumerate() {
            if i > 0 {
                out.push_str(&format!("\n{}\n", "─".repeat(40).dimmed()));
            }
            out.push_str(&t.render(OutputMode::Human));
        }
        if let Some(token) = &self.next_token {
            out.push_str(&format!("\n\n{}", format!("Next: {token}").dimmed()));
        }
        out
    }

    fn render_plain(&self) -> String {
        let mut lines: Vec<String> = self.tweets.iter().map(|t| t.render(OutputMode::Plain)).collect();
        if let Some(token) = &self.next_token {
            lines.push(format!("next_token\t{token}"));
        }
        lines.join("\n")
    }

    fn render_markdown(&self) -> String {
        let mut parts: Vec<String> = self.tweets.iter().map(|t| t.render(OutputMode::Markdown)).collect();
        if let Some(token) = &self.next_token {
            parts.push(format!("\n---\n*Next page: `{token}`*"));
        }
        parts.join("\n\n---\n\n")
    }
}

impl Renderable for UserList {
    fn render_human(&self) -> String {
        use colored::Colorize;
        let mut out = String::new();
        for (i, u) in self.users.iter().enumerate() {
            if i > 0 {
                out.push_str(&format!("\n{}\n", "─".repeat(40).dimmed()));
            }
            out.push_str(&u.render(OutputMode::Human));
        }
        if let Some(token) = &self.next_token {
            out.push_str(&format!("\n\n{}", format!("Next: {token}").dimmed()));
        }
        out
    }

    fn render_plain(&self) -> String {
        let mut lines: Vec<String> = self.users.iter().map(|u| u.render(OutputMode::Plain)).collect();
        if let Some(token) = &self.next_token {
            lines.push(format!("next_token\t{token}"));
        }
        lines.join("\n")
    }

    fn render_markdown(&self) -> String {
        let mut parts: Vec<String> = self.users.iter().map(|u| u.render(OutputMode::Markdown)).collect();
        if let Some(token) = &self.next_token {
            parts.push(format!("\n---\n*Next page: `{token}`*"));
        }
        parts.join("\n\n---\n\n")
    }
}

impl Renderable for MutationResult {
    fn render_human(&self) -> String {
        use colored::Colorize;
        let status = if self.success {
            "✓".green().to_string()
        } else {
            "✗".red().to_string()
        };
        let id_part = self
            .id
            .as_ref()
            .map(|id| format!(" (ID: {id})"))
            .unwrap_or_default();
        format!("{status} {}{id_part}", self.action)
    }

    fn render_plain(&self) -> String {
        let id = self.id.as_deref().unwrap_or("");
        format!("{}\t{}\t{}", self.action, self.success, id)
    }

    fn render_markdown(&self) -> String {
        let status = if self.success { "Success" } else { "Failed" };
        let id_part = self
            .id
            .as_ref()
            .map(|id| format!(" (`{id}`)"))
            .unwrap_or_default();
        format!("**{}**: {status}{id_part}", self.action)
    }
}

impl Renderable for AuthStatus {
    fn render_human(&self) -> String {
        use colored::Colorize;
        let status = if self.authenticated {
            "Authenticated".green().to_string()
        } else {
            "Not authenticated".red().to_string()
        };
        let mut out = format!("{status} via {}", self.method.bold());
        if let Some(uid) = &self.user_id {
            out.push_str(&format!("\nUser ID: {uid}"));
        }
        if let Some(exp) = &self.expires_at {
            out.push_str(&format!("\nExpires: {exp}"));
        }
        if let Some(scopes) = &self.scopes {
            out.push_str(&format!("\nScopes: {}", scopes.join(" ")));
        }
        out
    }

    fn render_plain(&self) -> String {
        let uid = self.user_id.as_deref().unwrap_or("");
        format!("{}\t{}\t{}", self.method, self.authenticated, uid)
    }

    fn render_markdown(&self) -> String {
        let status = if self.authenticated {
            "Authenticated"
        } else {
            "Not authenticated"
        };
        let mut out = format!("**{status}** via `{}`", self.method);
        if let Some(uid) = &self.user_id {
            out.push_str(&format!("\n- User ID: `{uid}`"));
        }
        if let Some(exp) = &self.expires_at {
            out.push_str(&format!("\n- Expires: {exp}"));
        }
        out
    }
}

impl Renderable for AuthLoginAction {
    fn render_human(&self) -> String {
        format!("{}: {}", self.action_required, self.url)
    }

    fn render_plain(&self) -> String {
        format!("{}\t{}", self.action_required, self.url)
    }

    fn render_markdown(&self) -> String {
        format!(
            "**{}**\n\n[Authorize here]({})",
            self.action_required, self.url
        )
    }
}
