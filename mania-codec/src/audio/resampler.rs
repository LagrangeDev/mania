pub mod rubato_resampler;

use crate::audio::{AudioResampleStream, ResampleSample};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioCodecResamplerError {}

pub trait AudioResampler<T: ResampleSample, U: ResampleSample> {
    fn resample(
        &self,
        input: &AudioResampleStream<T>,
    ) -> Result<AudioResampleStream<U>, AudioCodecResamplerError>;
}
