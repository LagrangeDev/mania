pub mod silk_encoder;

use crate::audio::{AudioEncodeStream, AudioResampleStream, EncodeSample, ResampleSample};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioCodecEncoderError {
    #[error("Silk encoder error: {0}")]
    SilkEncoderKnownError(#[from] silk_encoder::SilkError),
    #[error("Silk encoder error: {0}")]
    SilkEncoderUnknownError(i32),
}

pub trait AudioEncoder<T: EncodeSample + ResampleSample> {
    fn encode(
        &self,
        input: &AudioResampleStream<T>,
    ) -> Result<AudioEncodeStream<T>, AudioCodecEncoderError>;
}
