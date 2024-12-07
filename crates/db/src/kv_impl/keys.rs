use std::sync::LazyLock;

use kv::prelude::*;

static INDEX_NS_SEGMENT: LazyLock<StrictSlug> =
  LazyLock::new(|| StrictSlug::new("index".to_string()));
static MODEL_NS_SEGMENT: LazyLock<StrictSlug> =
  LazyLock::new(|| StrictSlug::new("model".to_string()));

pub(crate) fn model_base_key<M: model::Model>(id: &model::RecordId<M>) -> Key {
  let id_ulid: model::Ulid = (*id).into();
  Key::new_lazy(&MODEL_NS_SEGMENT)
    .with(StrictSlug::new(M::TABLE_NAME.to_string()))
    .with(StrictSlug::new(id_ulid.to_string()))
}

pub(crate) fn index_base_key<M: model::Model>(index_name: &str) -> Key {
  Key::new_lazy(&INDEX_NS_SEGMENT)
    .with(StrictSlug::new(M::TABLE_NAME.to_string()))
    .with(StrictSlug::new(index_name))
}
