//! Various stream tools.

mod counted_async_reader;
mod read_jump;

pub use tokio::io::AsyncRead;

pub use self::{
  counted_async_reader::CountedAsyncReader, read_jump::wrap_with_read_jump,
};

/// Trait alias for `Box<dyn AsyncReader + ...>`
pub type DynAsyncReader = Box<dyn AsyncRead + Send + Unpin + 'static>;
