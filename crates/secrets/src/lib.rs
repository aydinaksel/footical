use bitwarden::{
    Client,
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{SecretsClientExt, secrets::SecretGetRequest},
};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum SecretsError {
    #[error("BWS login failed: {message}")]
    LoginFailed { message: String },

    #[error("failed to fetch secret {env_var} ({secret_id}): {message}")]
    FetchFailed {
        env_var: String,
        secret_id: String,
        message: String,
    },

    #[error("invalid secret UUID {value}: {source}")]
    InvalidUuid {
        value: String,
        source: uuid::Error,
    },

    #[error("BWS access token file not found; set BWS_ACCESS_TOKEN_FILE to a readable path")]
    AccessTokenNotFound,
}

fn load_access_token() -> Result<String, SecretsError> {
    let path =
        std::env::var("BWS_ACCESS_TOKEN_FILE").map_err(|_| SecretsError::AccessTokenNotFound)?;

    let token = std::fs::read_to_string(&path)
        .map_err(|_| SecretsError::AccessTokenNotFound)?
        .trim()
        .to_owned();

    if token.is_empty() {
        return Err(SecretsError::AccessTokenNotFound);
    }

    Ok(token)
}

pub async fn inject_from_bws(
    secrets: &[(&str, &str)],
) -> Result<(), SecretsError> {
    let access_token = load_access_token()?;

    let client = Client::new(None);
    let response = client
        .auth()
        .login_access_token(&AccessTokenLoginRequest {
            access_token,
            state_file: None,
        })
        .await
        .map_err(|error| SecretsError::LoginFailed {
            message: error.to_string(),
        })?;

    if !response.authenticated {
        return Err(SecretsError::LoginFailed {
            message: "authenticated=false".to_owned(),
        });
    }

    let mut injected_count = 0;
    for &(env_var, secret_id) in secrets {
        if std::env::var(env_var).is_ok() {
            continue;
        }

        let uuid = Uuid::parse_str(secret_id).map_err(|source| {
            SecretsError::InvalidUuid {
                value: secret_id.to_owned(),
                source,
            }
        })?;

        let secret_response = client
            .secrets()
            .get(&SecretGetRequest { id: uuid })
            .await
            .map_err(|error| SecretsError::FetchFailed {
                env_var: env_var.to_owned(),
                secret_id: secret_id.to_owned(),
                message: error.to_string(),
            })?;

        unsafe { std::env::set_var(env_var, &secret_response.value) };
        injected_count += 1;
    }

    tracing::info!(
        name: "secrets.injected",
        injected_count,
        total_count = secrets.len(),
        "injected secrets from Bitwarden into environment",
    );

    Ok(())
}
