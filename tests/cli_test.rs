use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output() {
    Command::cargo_bin("ax")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Agent-first Twitter/X CLI"))
        .stdout(predicate::str::contains("tweet"))
        .stdout(predicate::str::contains("user"))
        .stdout(predicate::str::contains("self"))
        .stdout(predicate::str::contains("auth"));
}

#[test]
fn test_version() {
    Command::cargo_bin("ax")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ax "));
}

#[test]
fn test_tweet_help() {
    Command::cargo_bin("ax")
        .unwrap()
        .args(["tweet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("post"))
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("search"));
}

#[test]
fn test_auth_status_no_auth() {
    // With no auth configured, auth status should still work (shows unauthenticated)
    Command::cargo_bin("ax")
        .unwrap()
        .args(["auth", "status"])
        .env_remove("X_BEARER_TOKEN")
        .env_remove("X_API_KEY")
        .env_remove("X_API_SECRET")
        .env_remove("X_ACCESS_TOKEN")
        .env_remove("X_ACCESS_TOKEN_SECRET")
        .assert()
        .success()
        .stdout(predicate::str::contains("Not authenticated").or(predicate::str::contains("none")));
}

#[test]
fn test_auth_status_json_output() {
    Command::cargo_bin("ax")
        .unwrap()
        .args(["-o", "json", "auth", "status"])
        .env_remove("X_BEARER_TOKEN")
        .env_remove("X_API_KEY")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"authenticated\""));
}

#[test]
fn test_no_dna_json_default() {
    Command::cargo_bin("ax")
        .unwrap()
        .args(["auth", "status"])
        .env("NO_DNA", "1")
        .env_remove("X_BEARER_TOKEN")
        .env_remove("X_API_KEY")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"method\""));
}

#[test]
fn test_tweet_get_no_auth() {
    // Without auth, tweet get should fail with exit code 2 (auth error)
    Command::cargo_bin("ax")
        .unwrap()
        .args(["tweet", "get", "123"])
        .env_remove("X_BEARER_TOKEN")
        .env_remove("X_API_KEY")
        .env_remove("X_API_SECRET")
        .env_remove("X_ACCESS_TOKEN")
        .env_remove("X_ACCESS_TOKEN_SECRET")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn test_no_dna_error_json() {
    // In NO_DNA mode, errors should be JSON on stderr
    Command::cargo_bin("ax")
        .unwrap()
        .args(["tweet", "get", "123"])
        .env("NO_DNA", "1")
        .env_remove("X_BEARER_TOKEN")
        .env_remove("X_API_KEY")
        .env_remove("X_API_SECRET")
        .env_remove("X_ACCESS_TOKEN")
        .env_remove("X_ACCESS_TOKEN_SECRET")
        .assert()
        .failure()
        .stderr(predicate::str::contains("\"error\""))
        .stderr(predicate::str::contains("\"error_type\""))
        .stderr(predicate::str::contains("\"timestamp\""));
}

#[test]
fn test_auth_logout() {
    Command::cargo_bin("ax")
        .unwrap()
        .args(["auth", "logout"])
        .assert()
        .success();
}
