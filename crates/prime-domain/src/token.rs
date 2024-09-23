pub use models::Token;
use repos::TokenRepository;

/// The definition for the [`Token`] domain model service.
pub trait TokenService: Clone + Send + Sync + 'static {}

/// Canonical service for the [`Token`] domain model.
pub struct TokenServiceCanonical<R: TokenRepository> {
  cache_repo: R,
}

impl<R: TokenRepository> Clone for TokenServiceCanonical<R> {
  fn clone(&self) -> Self {
    Self {
      cache_repo: self.cache_repo.clone(),
    }
  }
}

impl<R: TokenRepository> TokenServiceCanonical<R> {
  /// Create a new instance of the canonical [`Token`] service.
  pub fn new(cache_repo: R) -> Self { Self { cache_repo } }
}

impl<R: TokenRepository> TokenService for TokenServiceCanonical<R> {}
