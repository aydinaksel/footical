use leptos::prelude::*;
#[cfg(feature = "ssr")]
use tracing::{event, Level};

#[cfg(feature = "ssr")]
const SESSION_COOKIE_NAME: &str = "footical_session";

#[server]
pub async fn login(password: String) -> Result<(), ServerFnError> {
    let expected = std::env::var("ADMIN_PASSWORD")
        .ok()
        .ok_or_else(|| ServerFnError::new("ADMIN_PASSWORD not configured"))?;

    if password != expected {
        event!(
            name: "admin.login.rejected",
            Level::WARN,
            "admin login rejected: wrong password",
        );
        return Err(ServerFnError::new("invalid password"));
    }

    let response_options = use_context::<leptos_axum::ResponseOptions>()
        .ok_or_else(|| ServerFnError::new("no response options"))?;

    let cookie_secret = std::env::var("COOKIE_SECRET").unwrap_or_else(|_| expected.clone());

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

    event!(
        name: "admin.login.success",
        Level::INFO,
        "admin signed in",
    );

    Ok(())
}

#[cfg(feature = "ssr")]
pub fn has_valid_session(cookie_header: Option<&str>) -> bool {
    let Some(cookie_header) = cookie_header else {
        return false;
    };

    let session_token = cookie_header
        .split(';')
        .map(|cookie| cookie.trim())
        .find_map(|cookie| cookie.strip_prefix(&format!("{SESSION_COOKIE_NAME}=")));

    let Some(session_token) = session_token else {
        return false;
    };

    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_default();
    let cookie_secret = std::env::var("COOKIE_SECRET").unwrap_or(admin_password);

    session_token == generate_session_token(&cookie_secret)
}

#[server]
pub async fn check_auth() -> Result<bool, ServerFnError> {
    let request = use_context::<axum::http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts"))?;

    Ok(has_valid_session(
        request
            .headers
            .get(axum::http::header::COOKIE)
            .and_then(|value| value.to_str().ok()),
    ))
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
