#[derive(Debug, clap::Subcommand)]
pub enum AuthAction {
    /// Log in via OAuth 2.0 PKCE flow
    Login {
        /// OAuth 2.0 scopes (space-separated)
        #[arg(long)]
        scopes: Option<String>,
        /// Local callback server port
        #[arg(long, default_value = "8477")]
        port: u16,
        /// Print auth URL and exit without starting local callback server
        #[arg(long)]
        no_browser: bool,
    },
    /// Complete non-interactive login with callback data
    Callback {
        /// Base64-encoded callback token (from oauth.cli.city)
        token: Option<String>,
        /// Authorization code (alternative to token)
        #[arg(long)]
        code: Option<String>,
        /// State parameter (alternative to token)
        #[arg(long)]
        state: Option<String>,
    },
    /// Show current authentication status
    Status,
    /// Log out and remove stored tokens
    Logout,
}
