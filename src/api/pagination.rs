/// Appends pagination query params to a URL being built.
pub fn apply_pagination_params(params: &mut Vec<(String, String)>, max_results: u32, next_token: &Option<String>) {
    params.push(("max_results".into(), max_results.to_string()));
    if let Some(token) = next_token {
        params.push(("pagination_token".into(), token.clone()));
    }
}
