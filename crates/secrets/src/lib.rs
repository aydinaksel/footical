use bitwarden::{
    Client,
    auth::login::AccessTokenLoginRequest,
    secrets_manager::{SecretsClientExt, secrets::SecretGetRequest},
};
use std::path::Path;
use tracing::{Level, event};
use uuid::Uuid;

const ACCESS_TOKEN_CREDENTIAL: &str = "bws-access-token";
const CREDENTIALS_DIRECTORY_VARIABLE: &str = "CREDENTIALS_DIRECTORY";

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
    InvalidUuid { value: String, source: uuid::Error },

    #[error(
        "no systemd credential directory; the unit must set \
         LoadCredential={ACCESS_TOKEN_CREDENTIAL}:<path>"
    )]
    CredentialsDirectoryNotFound,

    #[error("systemd credential {ACCESS_TOKEN_CREDENTIAL} is unreadable: {source}")]
    AccessTokenUnreadable { source: std::io::Error },

    #[error("systemd credential {ACCESS_TOKEN_CREDENTIAL} is empty")]
    AccessTokenEmpty,
}

fn find_access_token() -> Result<String, SecretsError> {
    let credentials_directory = std::env::var(CREDENTIALS_DIRECTORY_VARIABLE)
        .ok()
        .ok_or(SecretsError::CredentialsDirectoryNotFound)?;

    let token = std::fs::read_to_string(Path::new(&credentials_directory).join(ACCESS_TOKEN_CREDENTIAL))
        .map_err(|source| SecretsError::AccessTokenUnreadable { source })?
        .trim()
        .to_owned();

    if token.is_empty() {
        return Err(SecretsError::AccessTokenEmpty);
    }

    Ok(token)
}

pub async fn inject_from_bws(secrets: &[(&str, &str)]) -> Result<(), SecretsError> {
    let access_token = find_access_token()?;

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

    let mut injected_count: usize = 0;
    for &(env_var, secret_id) in secrets {
        if std::env::var(env_var).is_ok() {
            continue;
        }

        let uuid = Uuid::parse_str(secret_id).map_err(|source| SecretsError::InvalidUuid {
            value: secret_id.to_owned(),
            source,
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

        std::env::set_var(env_var, &secret_response.value);
        injected_count = injected_count.saturating_add(1);
    }

    event!(
        name: "secrets.injection.success",
        Level::INFO,
        secrets.injected_count = injected_count,
        secrets.requested_count = secrets.len(),
        "injected {{secrets.injected_count}} of {{secrets.requested_count}} secrets from Bitwarden",
    );

    Ok(())
}
