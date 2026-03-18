mod api;
mod auth;
mod cli;
mod config;
mod error;
mod output;

use std::process::ExitCode;

use clap::Parser;

use crate::api::types::{AuthStatus, MutationResult};
use crate::api::XClient;
use crate::auth::token_store;
use crate::cli::{Cli, Command};
use crate::config::RuntimeConfig;
use crate::error::AgentXError;
use crate::output::print_output;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let config = RuntimeConfig::from_cli(cli.output, cli.verbose);

    match run(cli.command, &config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            e.report(&config);
            e.exit_code()
        }
    }
}

async fn run(command: Command, config: &RuntimeConfig) -> Result<(), AgentXError> {
    match command {
        Command::Auth { action } => handle_auth(action, config).await,
        _ => {
            let auth = auth::resolve_auth()?;
            let client = XClient::new(auth)?;
            match command {
                Command::Tweet { action } => handle_tweet(action, &client, config).await,
                Command::User { action } => handle_user(action, &client, config).await,
                Command::SelfOps { action } => handle_self(action, &client, config).await,
                Command::Auth { .. } => unreachable!(),
            }
        }
    }
}

async fn handle_auth(
    action: cli::auth::AuthAction,
    config: &RuntimeConfig,
) -> Result<(), AgentXError> {
    match action {
        cli::auth::AuthAction::Login {
            scopes,
            port,
            no_browser,
        } => {
            let client_id = auth::oauth2::resolve_client_id();
            if config.no_dna || no_browser {
                auth::oauth2::login_noninteractive(&client_id, scopes.as_deref(), config.no_dna)?;
            } else {
                auth::oauth2::login(&client_id, scopes.as_deref(), port, config.no_dna).await?;
            }
            Ok(())
        }
        cli::auth::AuthAction::Callback { token, code, state } => {
            let (auth_code, auth_state) = if let Some(t) = token {
                auth::oauth2::decode_callback_token(&t)?
            } else {
                match (code, state) {
                    (Some(c), Some(s)) => (c, s),
                    _ => {
                        return Err(AgentXError::Auth(
                            "Provide either a base64 token or both --code and --state".to_string(),
                        ));
                    }
                }
            };
            auth::oauth2::complete_callback(&auth_code, &auth_state).await?;
            let result = MutationResult {
                action: "login".to_string(),
                success: true,
                id: None,
            };
            print_output(&result, config.output_mode);
            Ok(())
        }
        cli::auth::AuthAction::Status => {
            let status = match auth::resolve_auth() {
                Ok(provider) => {
                    let mut s = AuthStatus {
                        method: provider.method_name().to_string(),
                        authenticated: true,
                        user_id: None,
                        expires_at: None,
                        scopes: None,
                    };
                    // Add OAuth2-specific details if available
                    if let Ok(Some(tokens)) = token_store::load_tokens() {
                        s.expires_at = tokens.expires_at.map(|ts| {
                            chrono::DateTime::from_timestamp(ts, 0)
                                .map(|dt| dt.to_rfc3339())
                                .unwrap_or_else(|| ts.to_string())
                        });
                        if !tokens.scopes.is_empty() {
                            s.scopes = Some(tokens.scopes);
                        }
                    }
                    s
                }
                Err(_) => AuthStatus {
                    method: "none".to_string(),
                    authenticated: false,
                    user_id: None,
                    expires_at: None,
                    scopes: None,
                },
            };
            print_output(&status, config.output_mode);
            Ok(())
        }
        cli::auth::AuthAction::Logout => {
            token_store::delete_tokens()?;
            let result = MutationResult {
                action: "logout".to_string(),
                success: true,
                id: None,
            };
            print_output(&result, config.output_mode);
            Ok(())
        }
    }
}

async fn handle_tweet(
    action: cli::tweet::TweetAction,
    client: &XClient,
    config: &RuntimeConfig,
) -> Result<(), AgentXError> {
    use cli::tweet::TweetAction;

    match action {
        TweetAction::Get {
            id,
            fields,
            expansions,
        } => {
            let tweet = client
                .get_tweet(&id, fields.as_deref(), expansions.as_deref())
                .await?;
            print_output(&tweet, config.output_mode);
        }
        TweetAction::Post { text, media: _ } => {
            // TODO: media upload support
            let tweet = client.post_tweet(&text).await?;
            print_output(&tweet, config.output_mode);
        }
        TweetAction::Delete { id } => {
            let result = client.delete_tweet(&id).await?;
            print_output(&result, config.output_mode);
        }
        TweetAction::Reply { id, text } => {
            let tweet = client.reply_tweet(&id, &text).await?;
            print_output(&tweet, config.output_mode);
        }
        TweetAction::Quote { id, text } => {
            let tweet = client.quote_tweet(&id, &text).await?;
            print_output(&tweet, config.output_mode);
        }
        TweetAction::Search {
            query,
            max_results,
            next_token,
        } => {
            let list = client
                .search_tweets(&query, max_results, &next_token)
                .await?;
            print_output(&list, config.output_mode);
        }
        TweetAction::Metrics { id } => {
            let tweet = client.get_tweet_metrics(&id).await?;
            print_output(&tweet, config.output_mode);
        }
    }
    Ok(())
}

async fn handle_user(
    action: cli::user::UserAction,
    client: &XClient,
    config: &RuntimeConfig,
) -> Result<(), AgentXError> {
    use cli::user::UserAction;

    match action {
        UserAction::Get { username } => {
            let user = client.get_user(&username).await?;
            print_output(&user, config.output_mode);
        }
        UserAction::Timeline {
            user,
            max_results,
            next_token,
        } => {
            let list = client
                .get_user_timeline(&user, max_results, &next_token)
                .await?;
            print_output(&list, config.output_mode);
        }
        UserAction::Followers {
            user,
            max_results,
            next_token,
        } => {
            let list = client
                .get_user_followers(&user, max_results, &next_token)
                .await?;
            print_output(&list, config.output_mode);
        }
        UserAction::Following {
            user,
            max_results,
            next_token,
        } => {
            let list = client
                .get_user_following(&user, max_results, &next_token)
                .await?;
            print_output(&list, config.output_mode);
        }
    }
    Ok(())
}

async fn handle_self(
    action: cli::self_ops::SelfAction,
    client: &XClient,
    config: &RuntimeConfig,
) -> Result<(), AgentXError> {
    use cli::self_ops::SelfAction;

    match action {
        SelfAction::Mentions {
            max_results,
            next_token,
        } => {
            let list = client.get_mentions(max_results, &next_token).await?;
            print_output(&list, config.output_mode);
        }
        SelfAction::Bookmarks {
            max_results,
            next_token,
        } => {
            let list = client.get_bookmarks(max_results, &next_token).await?;
            print_output(&list, config.output_mode);
        }
        SelfAction::Like { id } => {
            let result = client.like_tweet(&id).await?;
            print_output(&result, config.output_mode);
        }
        SelfAction::Unlike { id } => {
            let result = client.unlike_tweet(&id).await?;
            print_output(&result, config.output_mode);
        }
        SelfAction::Retweet { id } => {
            let result = client.retweet(&id).await?;
            print_output(&result, config.output_mode);
        }
        SelfAction::Unretweet { id } => {
            let result = client.unretweet(&id).await?;
            print_output(&result, config.output_mode);
        }
        SelfAction::Bookmark { id } => {
            let result = client.bookmark_tweet(&id).await?;
            print_output(&result, config.output_mode);
        }
        SelfAction::Unbookmark { id } => {
            let result = client.unbookmark_tweet(&id).await?;
            print_output(&result, config.output_mode);
        }
    }
    Ok(())
}
