//! Provides prime-domain Services, the entry points for domain-specific
//! business logic.
//!
//! This is where our services are defined. A service is a domain-specific
//! business logic entrypoint. It has a single responsibility and has everything
//! necessary to manipulate a sub-category of the domain.
//!
//! All of the business logic for a given domain operation should be inside a
//! service method. Data should be validated and encapsulated before it gets to
//! the service.

mod cache;
mod entry;
mod store;
mod temp_storage;
mod token;

pub use hex;
pub use models;
pub use repos;

pub use self::{
  cache::{CacheService, CacheServiceCanonical, DynCacheService},
  entry::{DynEntryService, EntryService, EntryServiceCanonical},
  store::{DynStoreService, StoreService, StoreServiceCanonical},
  temp_storage::{
    DynTempStorageService, TempStorageService, TempStorageServiceCanonical,
  },
  token::{DynTokenService, TokenService, TokenServiceCanonical},
};
