//! Provides a generic `Retryable` that allows type-encoded errors to
//! become stateful errors.
use std::{fmt::Debug, future::Future, time::Duration};

use tokio::time::sleep;

use crate::Hexagonal;

/// A generic retryable type that allows for stateful errors.
pub struct Retryable<R, E>(Result<R, E>);

impl<R, E> Retryable<R, E>
where
  E: Debug + Send + Sync + 'static,
{
  /// Initializes a new `Retryable` with the given function, attempt limit, and
  /// delay.
  #[tracing::instrument(skip(func))]
  pub async fn init<Fut>(
    attempt_limit: u32,
    delay: Duration,
    func: impl Fn() -> Fut + 'static,
  ) -> Self
  where
    Fut: Future<Output = Result<R, E>> + 'static,
  {
    tracing::info!("attempting to init");
    let mut error = None;

    for attempt in 1..=attempt_limit {
      match func().await {
        Ok(value) => {
          return Retryable(Ok(value));
        }
        Err(err) => {
          tracing::warn!(
            "attempt to init #{attempt} failed with error: {err:?}",
          );
          error = Some(err);
          if attempt < attempt_limit {
            sleep(delay).await;
          }
        }
      }
    }

    Retryable(Err(error.unwrap()))
  }

  /// Returns the inner result.
  pub fn inner(&self) -> &Result<R, E> { &self.0 }
}

// impl healthreporter
#[health::async_trait]
impl<R, E> health::HealthReporter for Retryable<R, E>
where
  R: Hexagonal,
  E: Debug + Send + Sync + 'static,
{
  fn name(&self) -> &'static str { stringify!(Retryable) }
  async fn health_check(&self) -> health::ComponentHealth {
    match &self.0 {
      Ok(val) => {
        health::AdditiveComponentHealth::from_futures(vec![val.health_report()])
          .await
          .into()
      }
      Err(err) => {
        health::SingularComponentHealth::new(health::HealthStatus::Down(vec![
          health::FailureMessage::new(&format!("stateful error: {err:?}")),
        ]))
        .into()
      }
    }
  }
}
