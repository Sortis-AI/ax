#[derive(Debug, clap::Subcommand)]
pub enum UserAction {
    /// Get user by username
    Get {
        /// Username (without @)
        username: String,
    },
    /// Get a user's timeline
    Timeline {
        /// Username or user ID
        user: String,
        /// Maximum number of results (5-100)
        #[arg(long, default_value = "10")]
        max_results: u32,
        /// Pagination token
        #[arg(long)]
        next_token: Option<String>,
    },
    /// Get a user's followers
    Followers {
        /// Username or user ID
        user: String,
        /// Maximum number of results (1-1000)
        #[arg(long, default_value = "100")]
        max_results: u32,
        /// Pagination token
        #[arg(long)]
        next_token: Option<String>,
    },
    /// Get users a user is following
    Following {
        /// Username or user ID
        user: String,
        /// Maximum number of results (1-1000)
        #[arg(long, default_value = "100")]
        max_results: u32,
        /// Pagination token
        #[arg(long)]
        next_token: Option<String>,
    },
}
