macro_rules! fetchers {
  ($model:ty, $singular_fn_name:ident, $enumerate_fn_name:ident, $fetch_single_fn_name:ident, $plural_fn_name:ident) => {
    #[leptos::prelude::server]
    pub async fn $plural_fn_name(
    ) -> Result<Vec<$model>, leptos::prelude::ServerFnError> {
      let service: Option<_> = leptos::prelude::use_context();
      let service: prime_domain::DynPrimeDomainService =
        service.ok_or(leptos::prelude::ServerFnError::new(
          "`PrimeDomainService` is not available.",
        ))?;

      let models = service.$enumerate_fn_name().await.map_err(|e| {
        leptos::prelude::ServerFnError::new(format!(
          "Failed to enumerate {} models: {}",
          stringify!($model),
          e
        ))
      })?;
      Ok(models)
    }

    #[leptos::prelude::server]
    pub async fn $singular_fn_name(
      id: models::RecordId<$model>,
    ) -> Result<$model, leptos::prelude::ServerFnError> {
      let service: Option<_> = leptos::prelude::use_context();
      let service: prime_domain::DynPrimeDomainService =
        service.ok_or(leptos::prelude::ServerFnError::new(
          "`PrimeDomainService` is not available.",
        ))?;

      let model = service
        .$fetch_single_fn_name(id)
        .await
        .map_err(|e| {
          leptos::prelude::ServerFnError::new(format!(
            "Failed to fetch {} model: {}",
            stringify!($model),
            e
          ))
        })?
        .ok_or(leptos::prelude::ServerFnError::new(format!(
          "{} model not found.",
          stringify!($model)
        )))?;
      Ok(model)
    }
  };
}

fetchers!(
  models::Cache,
  fetch_cache,
  enumerate_caches,
  fetch_cache_by_id,
  fetch_all_caches
);
fetchers!(
  models::Entry,
  fetch_entry,
  enumerate_entries,
  fetch_entry_by_id,
  fetch_all_entries
);
fetchers!(
  models::Store,
  fetch_store,
  enumerate_stores,
  fetch_store_by_id,
  fetch_all_stores
);
fetchers!(
  models::Token,
  fetch_token,
  enumerate_tokens,
  fetch_token_by_id,
  fetch_all_tokens
);
