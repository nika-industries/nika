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

/// Fetches temp storage credentials from the environment.
pub fn get_temp_storage_creds(
) -> Result<core_types::StorageCredentials, TempStorageCredsError> {
  Ok(core_types::StorageCredentials::R2(
    core_types::R2StorageCredentials::Default {
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
    },
  ))
}

/// A temporary storage path.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TempStoragePath(pub PathBuf);
