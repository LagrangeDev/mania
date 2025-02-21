pub mod symphonia_decoder;

use crate::audio::{AudioResampleStream, AudioRwStream, DecodeSample};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioCodecDecoderError {}

pub trait AudioDecoder<T: DecodeSample> {
    fn decode(
        &self,
        input: AudioRwStream<T>,
    ) -> Result<AudioResampleStream<T>, AudioCodecDecoderError>;
}
