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
    },
    /// Show current authentication status
    Status,
    /// Log out and remove stored tokens
    Logout,
}
