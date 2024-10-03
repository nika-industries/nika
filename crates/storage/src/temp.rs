//! Temp storage.

use std::{path::PathBuf, str::FromStr};

use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

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
  /// [`models::StorageCredentials`].
  pub fn as_creds(&self) -> models::StorageCredentials {
    models::StorageCredentials::R2(models::R2StorageCredentials::Default {
      access_key:        self.access_key.clone(),
      secret_access_key: self.secret_access_key.clone(),
      endpoint:          self.endpoint.clone(),
      bucket:            self.bucket.clone(),
    })
  }
}

/// A temporary storage path.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempStoragePath(pub PathBuf);

impl TempStoragePath {
  /// Creates a new temporary storage path.
  pub fn new(path: PathBuf) -> Self { Self(path) }

  /// Creates a new random temporary storage path.
  pub fn new_random() -> Self {
    Self(PathBuf::from_str(&models::Ulid::new().to_string()).unwrap())
  }
}
