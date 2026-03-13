use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncSeek};

pub trait AsyncReadSeek: AsyncRead + AsyncSeek {}
impl<T: AsyncRead + AsyncSeek> AsyncReadSeek for T {}
pub trait AsyncPureStreamTrait: AsyncReadSeek + Send + Sync + Unpin {}
impl<T: AsyncReadSeek + Send + Sync + Unpin> AsyncPureStreamTrait for T {}
pub type AsyncPureStream = Box<dyn AsyncReadSeek + Send + Sync + Unpin>;
pub type AsyncStream = Arc<tokio::sync::Mutex<AsyncPureStream>>;
