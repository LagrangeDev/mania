pub mod silk_encoder;

use thiserror::Error;

use crate::audio::{AudioEncodeStream, AudioResampleStream, EncodeSample, ResampleSample};

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
