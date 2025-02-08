use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManiaError {
    #[error("An mania network error occurred: {0}")]
    NetworkError(#[from] std::io::Error),

    #[error("An mania error occurred: {0}")]
    GenericError(Cow<'static, str>),

    #[error("An mania internal event downcast error occurred")]
    InternalEventDowncastError,

    #[error("An mania internal business error occurred: {0}")]
    InternalBusinessError(#[from] crate::core::business::BusinessError),
}

pub type ManiaResult<T> = Result<T, ManiaError>;
