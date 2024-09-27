//! Temp storage.

use std::path::PathBuf;

use miette::IntoDiagnostic;
use serde::{Deserialize, Serialize};

/// Error type for [`get_temp_storage_creds`].
#[derive(thiserror::Error, Debug, miette::Diagnostic)]
#[error("failed to fetch temp storage credentials: {error}")]
pub struct TempStorageCredsError {
  #[diagnostic(transparent)]
  error: miette::Report,
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
        .map_err(|error| TempStorageCredsError { error })?,
      secret_access_key: std::env::var("R2_TEMP_SECRET_ACCESS_KEY")
        .into_diagnostic()
        .map_err(|error| TempStorageCredsError { error })?,
      endpoint:          std::env::var("R2_TEMP_ENDPOINT")
        .into_diagnostic()
        .map_err(|error| TempStorageCredsError { error })?,
      bucket:            std::env::var("R2_TEMP_BUCKET")
        .into_diagnostic()
        .map_err(|error| TempStorageCredsError { error })?,
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
