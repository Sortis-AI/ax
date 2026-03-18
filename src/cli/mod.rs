pub mod auth;
pub mod self_ops;
pub mod tweet;
pub mod user;

use clap::Parser;

use crate::output::OutputMode;

#[derive(Debug, Parser)]
#[command(name = "ax", version, about = "Agent-first Twitter/X CLI")]
pub struct Cli {
    /// Output format (json, plain, markdown, human)
    #[arg(short, long, global = true)]
    pub output: Option<OutputMode>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Tweet operations (post, get, delete, reply, quote, search, metrics)
    Tweet {
        #[command(subcommand)]
        action: tweet::TweetAction,
    },
    /// User operations (get, timeline, followers, following)
    User {
        #[command(subcommand)]
        action: user::UserAction,
    },
    /// Self-account operations (mentions, bookmarks, likes, retweets)
    #[command(name = "self")]
    SelfOps {
        #[command(subcommand)]
        action: self_ops::SelfAction,
    },
    /// Authentication (login, status, logout)
    Auth {
        #[command(subcommand)]
        action: auth::AuthAction,
    },
}
