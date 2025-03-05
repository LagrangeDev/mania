pub mod rubato_resampler;

use thiserror::Error;

use crate::audio::{AudioResampleStream, ResampleSample};

#[derive(Debug, Error)]
pub enum AudioCodecResamplerError {
    #[error("Rubato resampler error: {0}")]
    RubatoResampleError(#[from] rubato::ResampleError),
    #[error("Rubato resampler construction error: {0}")]
    RubatoResamplerConstructionError(#[from] rubato::ResamplerConstructionError),
}

pub trait AudioResampler<T: ResampleSample, U: ResampleSample> {
    fn resample(
        &self,
        input: &AudioResampleStream<T>,
    ) -> Result<AudioResampleStream<U>, AudioCodecResamplerError>;
}
