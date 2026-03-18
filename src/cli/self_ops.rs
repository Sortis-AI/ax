#[derive(Debug, clap::Subcommand)]
pub enum SelfAction {
    /// Get your recent mentions
    Mentions {
        /// Maximum number of results (5-100)
        #[arg(long, default_value = "10")]
        max_results: u32,
        /// Pagination token
        #[arg(long)]
        next_token: Option<String>,
    },
    /// Get your bookmarks
    Bookmarks {
        /// Maximum number of results (1-100)
        #[arg(long, default_value = "10")]
        max_results: u32,
        /// Pagination token
        #[arg(long)]
        next_token: Option<String>,
    },
    /// Like a tweet
    Like {
        /// Tweet ID
        id: String,
    },
    /// Unlike a tweet
    Unlike {
        /// Tweet ID
        id: String,
    },
    /// Retweet a tweet
    Retweet {
        /// Tweet ID
        id: String,
    },
    /// Undo a retweet
    Unretweet {
        /// Tweet ID
        id: String,
    },
    /// Bookmark a tweet
    Bookmark {
        /// Tweet ID
        id: String,
    },
    /// Remove a bookmark
    Unbookmark {
        /// Tweet ID
        id: String,
    },
}
