use std::fmt::Debug;

use hex::retryable::Retryable;

use crate::prelude::*;

impl<KV: KvTransactional, E: Debug + Send + Sync + 'static> KvTransactional
  for Retryable<KV, E>
{
  type OptimisticTransaction = KV::OptimisticTransaction;
  type PessimisticTransaction = KV::PessimisticTransaction;

  async fn begin_optimistic_transaction(
    &self,
  ) -> KvResult<Self::OptimisticTransaction> {
    let result = self.inner();
    match result {
      Ok(kv) => kv.begin_optimistic_transaction().await,
      Err(err) => Err(KvError::PlatformError(miette::miette!(
        "KV store is statefully errored: {err:?}"
      ))),
    }
  }
  async fn begin_pessimistic_transaction(
    &self,
  ) -> KvResult<Self::PessimisticTransaction> {
    let result = self.inner();
    match result {
      Ok(kv) => kv.begin_pessimistic_transaction().await,
      Err(err) => Err(KvError::PlatformError(miette::miette!(
        "KV store is statefully errored: {err:?}"
      ))),
    }
  }
}
