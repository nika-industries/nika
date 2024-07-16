use std::{
  fmt::{Debug, Display},
  marker::PhantomData,
  str::FromStr,
  sync::OnceLock,
};

use miette::{Context, Diagnostic, IntoDiagnostic};
use redis::{aio::MultiplexedConnection, AsyncCommands, Client, RedisError};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

/// The primary interface for defining tasks.
///
/// Semantically, implementing `Task` on an item means that the item is the
/// task's parameters. The `run()` method defines the logic of completing that
/// task.
#[async_trait::async_trait]
pub trait Task:
  Debug + Serialize + for<'a> Deserialize<'a> + Send + Sync + 'static
{
  /// The name of the task.
  const NAME: &'static str;

  /// The "success" return type of the task.
  type Response: Debug
    + Serialize
    + for<'a> Deserialize<'a>
    + Send
    + Sync
    + 'static;
  /// The "failure" return type of the task.
  type Error: Debug
    + Serialize
    + for<'a> Deserialize<'a>
    + Send
    + Sync
    + 'static;

  /// The run function for executing the task.
  async fn run(self) -> Result<Self::Response, Self::Error>;
}

/// Represents the status of a task.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Status<T: Task> {
  Pending,
  InProgress { worker_name: String },
  Completed(T::Response),
  Failed(T::Error),
  Panicked,
}

/// Represents the status of a task that has completed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FinishedStatus<T: Task> {
  Completed(T::Response),
  Failed(T::Error),
  Panicked,
}

/// A trait for IDs that can be used with a backend.
pub trait TaskId: Copy + Clone + Display + Debug + FromStr {
  fn new() -> Self;
}

impl TaskId for ulid::Ulid {
  fn new() -> Self { ulid::Ulid::new() }
}

/// A trait defining a task backend.
#[async_trait::async_trait]
pub trait Backend<T: Task> {
  type Id: TaskId;
  type Error: Diagnostic;

  async fn submit_task(&self, task: T) -> Result<Self::Id, Self::Error>;
  async fn get_status(
    &self,
    id: Self::Id,
  ) -> Result<Option<Status<T>>, Self::Error>;
  async fn consume(&self);
  async fn await_task(
    &self,
    id: Self::Id,
    poll_interval: Duration,
  ) -> Result<Option<FinishedStatus<T>>, Self::Error>;
}

/// A redis-based task backend.
#[derive(Clone)]
pub struct RedisBackend<T: Task> {
  worker_name: OnceLock<String>,
  conn:        MultiplexedConnection,
  _t:          PhantomData<T>,
}

impl<T: Task> RedisBackend<T> {
  pub async fn new() -> Self {
    RedisBackend {
      worker_name: OnceLock::new(),
      conn:        Client::open(std::env::var("REDIS_URL").unwrap())
        .into_diagnostic()
        .wrap_err("failed to open redis client")
        .unwrap()
        .get_multiplexed_async_connection()
        .await
        .into_diagnostic()
        .wrap_err("failed to get redis connection")
        .unwrap(),
      _t:          PhantomData,
    }
  }
}

/// The error type for [`RedisBackend`].
#[derive(Debug, thiserror::Error, Diagnostic)]
pub enum RedisBackendError {
  #[error("ser/de error: {0}")]
  SerdeJsonError(#[from] serde_json::Error),
  #[error("redis error: {0}")]
  RedisError(#[from] redis::RedisError),
}

fn task_data_key(name: impl Display, id: impl Display) -> String {
  format!("task:{name}:data:{id}")
}
fn task_status_key(name: impl Display, id: impl Display) -> String {
  format!("task:{name}:status:{id}")
}
fn task_queue_key(name: impl Display) -> String { format!("task:{name}:queue") }

#[async_trait::async_trait]
impl<T: Task> Backend<T> for RedisBackend<T> {
  type Id = ulid::Ulid;
  type Error = RedisBackendError;

  #[tracing::instrument(skip(self), fields(task_name = T::NAME))]
  async fn submit_task(&self, task: T) -> Result<Self::Id, Self::Error> {
    let task_id = Self::Id::new();
    let task_data_key = task_data_key(T::NAME, task_id.to_string());
    let task_status_key = task_status_key(T::NAME, task_id.to_string());
    let task_queue_key = task_queue_key(T::NAME);

    let task_data_ser = serde_json::to_string(&task)?;
    let task_status_ser = serde_json::to_string(&Status::<T>::Pending)?;

    tracing::info!("submitting task {}:{}", T::NAME, task_id.to_string());
    let mut conn = self.conn.clone();
    let _: () = conn.set(task_data_key, task_data_ser).await?;
    let _: () = conn.set(task_status_key, task_status_ser).await?;
    let _: () = conn.rpush(task_queue_key, task_id.to_string()).await?;
    Ok(task_id)
  }

  #[tracing::instrument(skip(self), fields(task_name = T::NAME))]
  async fn get_status(
    &self,
    task_id: Self::Id,
  ) -> Result<Option<Status<T>>, Self::Error> {
    let task_status_key = task_status_key(T::NAME, task_id.to_string());

    let mut conn = self.conn.clone();
    let task_status_ser: Option<String> = conn.get(task_status_key).await?;
    drop(conn);

    let Some(task_status_ser) = task_status_ser else {
      return Ok(None);
    };

    let task_status: Status<T> = serde_json::from_str(&task_status_ser)?;

    Ok(Some(task_status))
  }

  #[tracing::instrument(skip(self), fields(task_name = T::NAME))]
  async fn consume(&self) {
    let mut conn = self.conn.clone();
    let worker_name = self.worker_name.get_or_init(names::name);
    let task_queue_key = task_queue_key(T::NAME);

    tracing::info!("consuming {} tasks", T::NAME);
    loop {
      let task_id: Result<Option<String>, RedisError> =
        conn.lpop(&task_queue_key, None).await;
      tracing::info!("polling");
      let task_id = match task_id {
        Ok(Some(id)) => match Self::Id::from_str(&id) {
          Ok(id) => id,
          Err(_) => {
            tracing::warn!("popped bad ID from {task_queue_key:?}: {id}");
            sleep(Duration::from_millis(25)).await;
            continue;
          }
        },
        Ok(None) => {
          sleep(Duration::from_millis(25)).await;
          continue;
        }
        Err(e) => {
          tracing::error!(
            "failed to `blpop` from {task_queue_key:?}, continuing worker: {e}"
          );
          sleep(Duration::from_millis(25)).await;
          continue;
        }
      };

      tokio::spawn(run_task::<T>(
        task_id,
        self.conn.clone(),
        worker_name.clone(),
      ));
    }
  }

  #[tracing::instrument(skip(self), fields(task_name = T::NAME))]
  async fn await_task(
    &self,
    id: Self::Id,
    poll_interval: Duration,
  ) -> Result<Option<FinishedStatus<T>>, Self::Error> {
    loop {
      match self.get_status(id).await? {
        Some(Status::Completed(response)) => {
          return Ok(Some(FinishedStatus::Completed(response)));
        }
        Some(Status::Failed(error)) => {
          return Ok(Some(FinishedStatus::Failed(error)));
        }
        Some(Status::Panicked) => {
          return Ok(Some(FinishedStatus::Panicked));
        }
        None => {
          return Ok(None);
        }
        _ => (),
      }

      sleep(poll_interval).await;
    }
  }
}

#[tracing::instrument(skip(conn), fields(task_name = T::NAME))]
async fn run_task<T: Task>(
  task_id: impl TaskId,
  mut conn: MultiplexedConnection,
  worker_name: String,
) {
  let result: miette::Result<()> = async move {
    tracing::info!("running task");

    // assert that the status is `Status::Pending`
    // this is not necessary but good for debugging
    let task_status_key = task_status_key(T::NAME, task_id);
    let expected_status = serde_json::to_string(&Status::<T>::Pending)
      .into_diagnostic()
      .wrap_err("failed to serialize `Status`")?;
    let prev_status: Option<String> = conn
      .get(&task_status_key)
      .await
      .into_diagnostic()
      .wrap_err("failed to fetch previous status when popped from queue")?;
    match prev_status {
      Some(prev) if prev == expected_status => (),
      Some(prev) => tracing::warn!(
        "possible race condition: expected status `{expected_status:?}`, \
         found `{prev:?}`",
      ),
      None => tracing::warn!(
        "status did not exist for task {task_id} when popped from queue",
      ),
    };

    // write the new status to `Status::InProgress { worker_name }`
    let new_status = serde_json::to_string(&Status::<T>::InProgress {
      worker_name: worker_name.clone(),
    })
    .into_diagnostic()
    .wrap_err("failed to serialize `Status`")?;
    let _: () = conn
      .set(&task_status_key, new_status)
      .await
      .into_diagnostic()
      .wrap_err("failed to set task status when popped from queue")?;

    // fetch the params
    let task_data_key = task_data_key(T::NAME, task_id);
    let params: Option<String> = conn
      .get(&task_data_key)
      .await
      .into_diagnostic()
      .wrap_err_with(|| format!("failed to fetch task params"))?;
    let params: T = serde_json::from_str(
      &params.ok_or(miette::miette!("task params did not exist for task"))?,
    )
    .into_diagnostic()
    .wrap_err("failed to deserialize task params")?;

    let result = tokio::spawn(T::run(params)).await;

    let status = serde_json::to_string(&match result {
      Ok(Ok(response)) => Status::<T>::Completed(response),
      Ok(Err(task_error)) => Status::<T>::Failed(task_error),
      Err(_) => {
        tracing::error!("task panicked");
        Status::<T>::Panicked
      }
    })
    .into_diagnostic()
    .wrap_err("failed to serialize result status")?;
    let _: () = conn
      .set(&task_status_key, status.clone())
      .await
      .into_diagnostic()
      .wrap_err("failed to set task status when task completed")?;

    tracing::info!("finished task with status: {status:?}");

    Ok(())
  }
  .await;
  match result {
    Err(e) => tracing::error!("failed to run task: {e:?}"),
    _ => (),
  }
}
