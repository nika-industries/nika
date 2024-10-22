//! Provides [`KvTransactionExt`] trait.

use miette::{Context, Result};
use tracing::instrument;

use crate::KvTransaction;

/// Extension trait for [`KvTransaction`].
#[async_trait::async_trait]
pub trait KvTransactionExt:
  KvTransaction + Send + Sync + 'static + Sized
{
  /// Rollback the transaction, consuming it.
  #[instrument(skip(self))]
  async fn to_rollback(mut self) -> Result<()> {
    self
      .rollback()
      .await
      .context("failed to rollback transaction")
  }

  /// Rollback the transaction with an error, consuming it.
  #[instrument(skip(self, error, context))]
  async fn to_rollback_with_error(
    self,
    error: miette::Report,
    context: &'static str,
  ) -> miette::Report {
    if let Err(e) = self.to_rollback().await {
      tracing::error!("failed to rollback transaction: {:?}", e);
      return e;
    }
    let e = error.wrap_err(context);
    tracing::error!("unrecoverable rollback: {:?}", e);
    e
  }

  /// Commit the transaction, consuming it.
  #[instrument(skip(self))]
  async fn to_commit(mut self) -> Result<()> {
    if let Err(e) = self.commit().await.context("failed to commit transaction")
    {
      tracing::error!("failed to commit transaction: {:?}", e);
      Err(e)?;
    }
    Ok(())
  }
}

impl<T> KvTransactionExt for T where T: KvTransaction + Send + Sync + 'static {}
