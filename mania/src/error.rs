use std::borrow::Cow;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("network error: {0}")]
    NetworkError(#[from] std::io::Error),

    #[error("invalid server response: {0}")]
    InvalidServerResponse(Cow<'static, str>),
}

pub type Result<T> = std::result::Result<T, Error>;
