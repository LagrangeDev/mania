use crate::audio::encoder::{AudioCodecEncoderError, AudioEncoder};
use crate::audio::{AudioEncodeStream, AudioResampleStream, ResampleSample};
use bytes::BufMut;
use num_enum::TryFromPrimitive;
use silk_sys::{SKP_Silk_SDK_Encode, SKP_Silk_SDK_Get_Encoder_Size, SKP_Silk_SDK_InitEncoder};
use std::ffi::{c_int, c_void};
use std::fmt;
use std::marker::PhantomData;
use thiserror::Error;

#[repr(i32)]
#[derive(Debug, Error, TryFromPrimitive)]
pub enum SilkError {
    EncInputInvalidNoOfSamples = -1,
    EncFsNotSupported = -2,
    EncPacketSizeNotSupported = -3,
    EncPayloadBufTooShort = -4,
    EncInvalidLossRate = -5,
    EncInvalidComplexitySetting = -6,
    EncInvalidInbandFecSetting = -7,
    EncInvalidDtxSetting = -8,
    EncInternalError = -9,
    DecInvalidSamplingFrequency = -10,
    DecPayloadTooLarge = -11,
    DecPayloadError = -12,
}

impl fmt::Display for SilkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct SilkEncoder<U: ResampleSample> {
    bit_rate: u32,
    _phantom: PhantomData<U>,
}

impl<U: ResampleSample> SilkEncoder<U> {
    pub fn new(bit_rate: u32) -> Self {
        Self {
            bit_rate,
            _phantom: PhantomData,
        }
    }
}

macro_rules! fast_unsafe_check {
    ($call:expr) => {{
        let code = unsafe { $call };
        if code != 0 {
            match SilkError::try_from(code) {
                Ok(err) => return Err(AudioCodecEncoderError::SilkEncoderKnownError(err)),
                Err(_) => return Err(AudioCodecEncoderError::SilkEncoderUnknownError(code)),
            }
        }
    }};
}

// ref: https://github.com/lz1998/silk-rs/blob/main/src/encode.rs
impl AudioEncoder<i16> for SilkEncoder<i16> {
    fn encode(
        &self,
        input: &AudioResampleStream<i16>,
    ) -> Result<AudioEncodeStream<i16>, AudioCodecEncoderError> {
        let (stream, info) = (&input.stream, &input.info);
        let stream: &[u8] =
            unsafe { std::slice::from_raw_parts(stream.as_ptr() as *const u8, stream.len() * 2) };
        let sample_rate = info.sample_rate;
        let bit_rate = self.bit_rate as c_int;
        let enc_control = silk_sys::SKP_SILK_SDK_EncControlStruct {
            API_sampleRate: sample_rate as c_int,
            maxInternalSampleRate: 24000,
            packetSize: ((20 * sample_rate) / 1000) as c_int,
            bitRate: bit_rate,
            packetLossPercentage: 0,
            complexity: 2,
            useInBandFEC: 0,
            useDTX: 0,
        };
        let mut enc_status = silk_sys::SKP_SILK_SDK_EncControlStruct {
            API_sampleRate: 0,
            maxInternalSampleRate: 0,
            packetSize: 0,
            bitRate: bit_rate,
            packetLossPercentage: 0,
            complexity: 0,
            useInBandFEC: 0,
            useDTX: 0,
        };

        let mut encoder_size = 0;
        fast_unsafe_check!(SKP_Silk_SDK_Get_Encoder_Size(&mut encoder_size));

        let mut encoder = vec![0u8; encoder_size as usize];
        fast_unsafe_check!(SKP_Silk_SDK_InitEncoder(
            encoder.as_mut_ptr() as *mut c_void,
            &mut enc_status,
        ));

        let mut result = vec![];
        result.put_u8(b'\x02');
        result.extend_from_slice(b"#!SILK_V3");

        let frame_size = sample_rate as usize / 1000 * 40;
        let mut output_size = 1250i16;
        let mut buf = vec![0u8; output_size as usize];
        for chunk in stream.chunks(frame_size) {
            output_size = 1250;
            if chunk.len() < frame_size {
                break;
            }
            fast_unsafe_check!(SKP_Silk_SDK_Encode(
                encoder.as_mut_ptr() as *mut c_void,
                &enc_control,
                chunk.as_ptr() as *const i16,
                chunk.len() as i32 / 2,
                buf.as_mut_ptr(),
                &mut output_size,
            ));
            result.put_i16_le(output_size);
            result.extend_from_slice(&buf[0..output_size as usize]);
        }

        let res = AudioEncodeStream {
            stream: result,
            info: info.clone(),
        };

        Ok(res)
    }
}
