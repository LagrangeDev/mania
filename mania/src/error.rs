use crate::business::BusinessError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManiaError {
    #[error("A business error occurred: {0}")]
    BusinessError(#[from] BusinessError),
}

pub type ManiaResult<T> = Result<T, ManiaError>;
