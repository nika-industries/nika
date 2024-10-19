macro_rules! fetchers {
  ($service:ty, $model:ty, $singular_fn_name:ident, $plural_fn_name:ident) => {
    #[leptos::prelude::server]
    pub async fn $plural_fn_name(
    ) -> Result<Vec<$model>, leptos::prelude::ServerFnError> {
      let service: Option<_> = leptos::prelude::use_context();
      let service: $service =
        service.ok_or(leptos::prelude::ServerFnError::new(stringify!(
          $service,
          " service is not available."
        )))?;

      let ids = service.enumerate_models().await.map_err(|e| {
        leptos::prelude::ServerFnError::new(format!(
          "Failed to enumerate {} models: {}",
          stringify!($model),
          e
        ))
      })?;
      Ok(ids)
    }

    #[leptos::prelude::server]
    pub async fn $singular_fn_name(
      id: models::RecordId<$model>,
    ) -> Result<$model, leptos::prelude::ServerFnError> {
      let service: Option<_> = leptos::prelude::use_context();
      let service: $service =
        service.ok_or(leptos::prelude::ServerFnError::new(stringify!(
          $service,
          " service is not available."
        )))?;

      let model = service
        .fetch(id)
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
  prime_domain::DynCacheService,
  models::Cache,
  fetch_cache,
  fetch_all_caches
);
fetchers!(
  prime_domain::DynEntryService,
  models::Entry,
  fetch_entry,
  fetch_all_entries
);
fetchers!(
  prime_domain::DynStoreService,
  models::Store,
  fetch_store,
  fetch_all_stores
);
fetchers!(
  prime_domain::DynTokenService,
  models::Token,
  fetch_token,
  fetch_all_tokens
);
