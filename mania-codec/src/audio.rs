use crate::audio::decoder::{AudioCodecDecoderError, AudioDecoder};
use crate::audio::encoder::{AudioCodecEncoderError, AudioEncoder};
use crate::audio::resampler::{AudioCodecResamplerError, AudioResampler};
use num_traits::FromPrimitive;
use std::io::{Read, Seek};
use std::marker::PhantomData;
use symphonia::core::conv::{FromSample, IntoSample};
use symphonia::core::sample::{i24, u24};
use thiserror::Error;

pub mod decoder;
pub mod encoder;
pub mod resampler;

#[derive(Debug, Error)]
pub enum AudioCodecError {
    #[error("Decode error {0}")]
    DecodeError(#[from] AudioCodecDecoderError),
    #[error("Resample error {0}")]
    ResampleError(#[from] AudioCodecResamplerError),
    #[error("Encode error {0}")]
    EncodeError(#[from] AudioCodecEncoderError),
}

pub trait Sample: Copy + Send + Sync {}
impl Sample for u8 {}
impl Sample for i16 {}
impl Sample for i32 {}
impl Sample for f32 {}

pub trait ResampleSample: Sample + IntoSample<f32> {}
impl ResampleSample for i16 {}
impl ResampleSample for f32 {}

pub trait DecodeSample:
    FromSample<u8>
    + FromSample<u16>
    + FromSample<u24>
    + FromSample<u32>
    + FromSample<i8>
    + FromSample<i16>
    + FromSample<i24>
    + FromSample<i32>
    + FromSample<f32>
    + FromSample<f64>
    + FromPrimitive
    + ResampleSample
{
}
impl DecodeSample for f32 {}

pub trait EncodeSample: Sample {}
impl EncodeSample for i16 {}
impl EncodeSample for u8 {}

pub trait RSStream: Read + Seek + Send + Sync {
    fn is_seekable(&self) -> bool;
    fn byte_len(&self) -> Option<u64>;
}

impl RSStream for std::fs::File {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        self.metadata().ok().map(|m| m.len())
    }
}

#[derive(Debug, Clone)]
pub struct AudioInfo<T: Sample> {
    pub sample_rate: u32,
    pub channels: u16,
    _phantom: PhantomData<T>,
}

impl<T: Sample> AudioInfo<T> {
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        Self {
            sample_rate,
            channels,
            _phantom: PhantomData,
        }
    }
}

pub struct AudioRwStream<T: ResampleSample> {
    pub stream: Box<dyn RSStream>,
    pub info: Option<AudioInfo<T>>,
}

impl<T: ResampleSample> AudioRwStream<T> {
    pub fn new(stream: Box<dyn RSStream>) -> Self {
        Self {
            stream,
            info: None, // cannot get info now
        }
    }

    pub fn decode<D: AudioDecoder<T>>(
        self,
        decoder: D,
    ) -> Result<AudioResampleStream<T>, AudioCodecDecoderError>
    where
        T: DecodeSample,
    {
        decoder.decode(self)
    }
}

pub struct AudioResampleStream<T: ResampleSample> {
    pub stream: Vec<T>,
    pub info: AudioInfo<T>,
}

impl<T: ResampleSample> AudioResampleStream<T> {
    pub fn resample<U: ResampleSample, R: AudioResampler<T, U>>(
        self,
        resampler: R,
    ) -> Result<AudioResampleStream<U>, AudioCodecResamplerError> {
        resampler.resample(&self)
    }

    pub fn encode<E: AudioEncoder<T>>(
        self,
        encoder: E,
    ) -> Result<AudioEncodeStream<T>, AudioCodecEncoderError>
    where
        T: EncodeSample,
    {
        encoder.encode(&self)
    }
}

pub struct AudioEncodeStream<T: EncodeSample> {
    pub stream: Vec<u8>,
    pub info: AudioInfo<T>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::audio::decoder::symphonia_decoder::SymphoniaDecoder;
    use crate::audio::encoder::silk_encoder::SilkEncoder;
    use crate::audio::resampler::rubato_resampler::RubatoResampler;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_pipeline() -> Result<(), AudioCodecError> {
        let pipeline = AudioRwStream::new(Box::new(File::open("resource/test.mp3").unwrap()))
            .decode(SymphoniaDecoder::<f32>::new())?
            .resample(RubatoResampler::<i16>::new(24000))?
            .encode(SilkEncoder::new(30000))?;
        let mut file = File::create("resource/test.silk").unwrap();
        file.write_all(&pipeline.stream).unwrap();
        Ok(())
    }
}
