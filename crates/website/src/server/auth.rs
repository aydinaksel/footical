use leptos::prelude::*;

#[allow(dead_code)]
const SESSION_COOKIE_NAME: &str = "footical_session";

#[server]
pub async fn login(password: String) -> Result<(), ServerFnError> {
    let expected = std::env::var("ADMIN_PASSWORD")
        .map_err(|_| ServerFnError::new("ADMIN_PASSWORD not configured"))?;

    if password != expected {
        return Err(ServerFnError::new("invalid password"));
    }

    let response_options = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("no response options"))?;

    let cookie_secret = std::env::var("COOKIE_SECRET")
        .unwrap_or_else(|_| expected.clone());

    let token = generate_session_token(&cookie_secret);
    let cookie = format!(
        "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=86400",
        SESSION_COOKIE_NAME, token,
    );
    response_options.append_header(
        axum::http::HeaderName::from_static("set-cookie"),
        axum::http::HeaderValue::from_str(&cookie)
            .map_err(|error| ServerFnError::new(error.to_string()))?,
    );

    Ok(())
}

#[server]
pub async fn check_auth() -> Result<bool, ServerFnError> {
    let request = use_context::<axum::http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts"))?;

    let cookie_header = match request.headers.get("cookie") {
        Some(value) => value.to_str().unwrap_or(""),
        None => return Ok(false),
    };

    let session_value = cookie_header
        .split(';')
        .map(|cookie| cookie.trim())
        .find_map(|cookie| cookie.strip_prefix(&format!("{SESSION_COOKIE_NAME}=")));

    let Some(token) = session_value else {
        return Ok(false);
    };

    let expected_password = std::env::var("ADMIN_PASSWORD").unwrap_or_default();
    let cookie_secret = std::env::var("COOKIE_SECRET").unwrap_or(expected_password.clone());
    let expected_token = generate_session_token(&cookie_secret);

    Ok(token == expected_token)
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    let response_options = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("no response options"))?;

    let cookie = format!(
        "{}=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0",
        SESSION_COOKIE_NAME,
    );
    response_options.append_header(
        axum::http::HeaderName::from_static("set-cookie"),
        axum::http::HeaderValue::from_str(&cookie)
            .map_err(|error| ServerFnError::new(error.to_string()))?,
    );

    Ok(())
}

#[cfg(feature = "ssr")]
fn generate_session_token(secret: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    secret.hash(&mut hasher);
    "footical_admin".hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
