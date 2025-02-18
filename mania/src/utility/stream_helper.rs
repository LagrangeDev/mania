use std::pin::Pin;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::Mutex;

pub async fn mut_stream_ctx<T, IR, E, F>(lock: &Mutex<T>, f: F) -> Result<IR, E>
where
    T: Send + 'static,
    F: for<'a> FnOnce(&'a mut T) -> Pin<Box<dyn Future<Output = Result<IR, E>> + Send + 'a>>,
{
    let mut guard = lock.lock().await;
    f(&mut *guard).await
}

pub async fn stream_pipeline<S, F>(stream: &mut S, mut processor: F) -> io::Result<()>
where
    S: AsyncRead + Unpin,
    F: FnMut(&[u8]),
{
    let mut buf = [0_u8; 8192];
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        processor(&buf[..n]);
    }
    Ok(())
}
