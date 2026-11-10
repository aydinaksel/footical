use std::path::PathBuf;

pub use secrecy::{ExposeSecret, SecretString};

const CREDENTIALS_DIRECTORY_VARIABLE: &str = "CREDENTIALS_DIRECTORY";

#[derive(Debug, thiserror::Error)]
pub enum CredentialsError {
    #[error(
        "${CREDENTIALS_DIRECTORY_VARIABLE} is not set, so this process was started without \
         LoadCredential= and holds no credentials"
    )]
    DirectoryNotSet,

    #[error("credential {name} was not loaded into {}: {source}", directory.display())]
    Unreadable {
        name: String,
        directory: PathBuf,
        source: std::io::Error,
    },

    #[error("credential {name} in {} is empty", directory.display())]
    Empty { name: String, directory: PathBuf },
}

#[derive(Clone, Debug)]
pub struct Credentials {
    directory: PathBuf,
}

impl Credentials {
    pub fn from_systemd() -> Result<Self, CredentialsError> {
        std::env::var_os(CREDENTIALS_DIRECTORY_VARIABLE)
            .map(|directory| Self::in_directory(PathBuf::from(directory)))
            .ok_or(CredentialsError::DirectoryNotSet)
    }

    pub fn in_directory(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn read(&self, name: &str) -> Result<SecretString, CredentialsError> {
        let contents = std::fs::read_to_string(self.directory.join(name)).map_err(|source| {
            CredentialsError::Unreadable {
                name: name.to_owned(),
                directory: self.directory.clone(),
                source,
            }
        })?;

        let value = contents.trim_end_matches(['\n', '\r']);
        if value.is_empty() {
            return Err(CredentialsError::Empty {
                name: name.to_owned(),
                directory: self.directory.clone(),
            });
        }

        Ok(SecretString::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::{Credentials, CredentialsError, ExposeSecret, SecretString};

    const STAGED_VALUE: &str = "7f3c0d9e-staged-credential-value";
    const CREDENTIAL_NAME: &str = "admin-password";

    fn read_staged(
        label: &str,
        files: &[(&str, &str)],
    ) -> Option<Result<SecretString, CredentialsError>> {
        let directory = std::env::temp_dir().join(format!(
            "footical-credentials-{}-{label}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).ok()?;
        for (name, contents) in files {
            std::fs::write(directory.join(name), contents).ok()?;
        }
        let result = Credentials::in_directory(&directory).read(CREDENTIAL_NAME);
        std::fs::remove_dir_all(&directory).ok()?;
        Some(result)
    }

    #[test]
    fn reads_a_staged_credential_without_its_trailing_newline() {
        let staged = format!("{STAGED_VALUE}\n");
        let value = read_staged("trailing-newline", &[(CREDENTIAL_NAME, &staged)])
            .and_then(Result::ok)
            .map(|secret| secret.expose_secret().to_owned());

        assert_eq!(value.as_deref(), Some(STAGED_VALUE));
    }

    #[test]
    fn a_credential_that_was_not_loaded_is_an_error_naming_it() {
        let result = read_staged("missing", &[]);

        assert!(matches!(
            result,
            Some(Err(CredentialsError::Unreadable { ref name, .. })) if name == CREDENTIAL_NAME
        ));
    }

    #[test]
    fn an_empty_credential_is_an_error() {
        let result = read_staged("empty", &[(CREDENTIAL_NAME, "\n")]);

        assert!(matches!(result, Some(Err(CredentialsError::Empty { .. }))));
    }

    #[test]
    fn a_read_credential_never_appears_in_debug_output() {
        let rendered = read_staged("debug", &[(CREDENTIAL_NAME, STAGED_VALUE)])
            .and_then(Result::ok)
            .map(|secret| format!("{secret:?}"));

        assert!(rendered.is_some_and(|text| !text.contains(STAGED_VALUE)));
    }
}
