#[derive(Debug, clap::Subcommand)]
pub enum TweetAction {
    /// Post a new tweet
    Post {
        /// Tweet text content
        text: String,
        /// Paths to media files to attach
        #[arg(long)]
        media: Vec<String>,
    },
    /// Get a tweet by ID
    Get {
        /// Tweet ID
        id: String,
        /// Comma-separated tweet fields
        #[arg(long)]
        fields: Option<String>,
        /// Comma-separated expansions
        #[arg(long)]
        expansions: Option<String>,
    },
    /// Delete a tweet by ID
    Delete {
        /// Tweet ID
        id: String,
    },
    /// Reply to a tweet
    Reply {
        /// Tweet ID to reply to
        id: String,
        /// Reply text
        text: String,
    },
    /// Quote a tweet
    Quote {
        /// Tweet ID to quote
        id: String,
        /// Quote text
        text: String,
    },
    /// Search tweets
    Search {
        /// Search query
        query: String,
        /// Maximum number of results (10-100)
        #[arg(long, default_value = "10")]
        max_results: u32,
        /// Pagination token
        #[arg(long)]
        next_token: Option<String>,
    },
    /// Get tweet metrics
    Metrics {
        /// Tweet ID
        id: String,
    },
}
