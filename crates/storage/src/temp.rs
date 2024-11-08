//! Temp storage.

use miette::IntoDiagnostic;

/// Error type for [`TempStorageCreds::new_from_env()`].
#[derive(thiserror::Error, Debug, miette::Diagnostic)]
pub enum TempStorageCredsError {
  /// An environment variable is missing.
  #[error("failed to read environment variable: {0:?}")]
  MissingEnvVar(String),
}

/// Temporary storage credentials.
#[derive(Clone, Debug)]
pub struct TempStorageCreds {
  access_key:        String,
  secret_access_key: String,
  endpoint:          String,
  bucket:            String,
}

impl TempStorageCreds {
  /// Creates a new set of temporary storage credentials.
  pub fn new(
    access_key: String,
    secret_access_key: String,
    endpoint: String,
    bucket: String,
  ) -> Self {
    Self {
      access_key,
      secret_access_key,
      endpoint,
      bucket,
    }
  }

  /// Creates a new set of temporary storage credentials from the environment.
  pub fn new_from_env() -> Result<Self, TempStorageCredsError> {
    Ok(Self {
      access_key:        std::env::var("R2_TEMP_ACCESS_KEY")
        .into_diagnostic()
        .map_err(|_| {
          TempStorageCredsError::MissingEnvVar("R2_TEMP_ACCESS_KEY".into())
        })?,
      secret_access_key: std::env::var("R2_TEMP_SECRET_ACCESS_KEY")
        .into_diagnostic()
        .map_err(|_| {
          TempStorageCredsError::MissingEnvVar(
            "R2_TEMP_SECRET_ACCESS_KEY".into(),
          )
        })?,
      endpoint:          std::env::var("R2_TEMP_ENDPOINT")
        .into_diagnostic()
        .map_err(|_| {
          TempStorageCredsError::MissingEnvVar("R2_TEMP_ENDPOINT".into())
        })?,
      bucket:            std::env::var("R2_TEMP_BUCKET")
        .into_diagnostic()
        .map_err(|_| {
          TempStorageCredsError::MissingEnvVar("R2_TEMP_BUCKET".into())
        })?,
    })
  }

  /// Converts the temporary storage credentials to a
  /// [`dvf::StorageCredentials`].
  pub fn as_creds(&self) -> dvf::StorageCredentials {
    dvf::StorageCredentials::R2(dvf::R2StorageCredentials::Default {
      access_key:        self.access_key.clone(),
      secret_access_key: self.secret_access_key.clone(),
      endpoint:          self.endpoint.clone(),
      bucket:            self.bucket.clone(),
    })
  }
}
