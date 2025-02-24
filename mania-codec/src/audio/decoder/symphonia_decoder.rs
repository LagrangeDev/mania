use crate::audio::decoder::{AudioCodecDecoderError, AudioDecoder};
use crate::audio::{AudioInfo, AudioResampleStream, AudioRwStream, DecodeSample, RSStream};
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use symphonia::core::audio::{Audio, GenericAudioBufferRef};
use symphonia::core::codecs::audio::AudioDecoderOptions;
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, TrackType};
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;

struct RSStreamAdapter {
    inner: Box<dyn RSStream>,
    is_seekable: bool,
    byte_len: Option<u64>,
}

impl Read for RSStreamAdapter {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl Seek for RSStreamAdapter {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl RSStream for RSStreamAdapter {
    fn is_seekable(&self) -> bool {
        self.is_seekable
    }

    fn byte_len(&self) -> Option<u64> {
        self.byte_len
    }
}

impl MediaSource for RSStreamAdapter {
    fn is_seekable(&self) -> bool {
        self.is_seekable
    }

    fn byte_len(&self) -> Option<u64> {
        self.byte_len
    }
}

pub struct SymphoniaDecoder<T: DecodeSample> {
    _phantom: PhantomData<T>,
}

impl<T: DecodeSample> SymphoniaDecoder<T> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: DecodeSample> Default for SymphoniaDecoder<T> {
    fn default() -> Self {
        Self::new()
    }
}

// FIXME: It is now just a rigid conversion to mono (in the form of [0])!
fn conv<S, T>(samples: &mut Vec<S>, data: &symphonia::core::audio::AudioBuffer<T>)
where
    T: symphonia::core::audio::sample::Sample,
    S: symphonia::core::audio::conv::FromSample<T>,
{
    samples.extend(
        data.plane(0)
            .unwrap_or(&[]) // TODO: avoid unwrap_or (?
            .iter()
            .map(|v| S::from_sample(*v)),
    );
}

// ref: https://github.com/kyutai-labs/hibiki/blob/main/hibiki-rs/src/audio_io.rs
impl<T: DecodeSample> AudioDecoder<T> for SymphoniaDecoder<T> {
    fn decode(
        &self,
        input: AudioRwStream<T>,
    ) -> Result<AudioResampleStream<T>, AudioCodecDecoderError> {
        let (stream, _) = (input.stream, input.info);
        let is_seekable = stream.is_seekable();
        let byte_len = stream.byte_len();
        let adapter = RSStreamAdapter {
            inner: stream,
            is_seekable,
            byte_len,
        };
        let mss = MediaSourceStream::new(Box::new(adapter), Default::default());
        let hint = Hint::new();
        let fmt_opts: FormatOptions = Default::default();
        let meta_opts: MetadataOptions = Default::default();
        let dec_opts: AudioDecoderOptions = Default::default();
        let mut format = symphonia::default::get_probe().probe(&hint, mss, fmt_opts, meta_opts)?;
        let track = format.default_track(TrackType::Audio).ok_or(
            AudioCodecDecoderError::SymphoniaUnknownError("no audio track".to_string()),
        )?;
        let mut decoder = symphonia::default::get_codecs().make_audio_decoder(
            track
                .codec_params
                .as_ref()
                .ok_or(AudioCodecDecoderError::SymphoniaUnknownError(
                    "codec parameters missing".to_string(),
                ))?
                .audio()
                .ok_or(AudioCodecDecoderError::SymphoniaUnknownError(
                    "audio parameters missing".to_string(),
                ))?,
            &dec_opts,
        )?;
        let track_id = track.id;
        let sample_rate = track
            .codec_params
            .as_ref()
            .ok_or(AudioCodecDecoderError::SymphoniaUnknownError(
                "codec parameters missing".to_string(),
            ))?
            .audio()
            .ok_or(AudioCodecDecoderError::SymphoniaUnknownError(
                "audio parameters missing".to_string(),
            ))?
            .sample_rate
            .ok_or(AudioCodecDecoderError::SymphoniaUnknownError(
                "sample rate missing".to_string(),
            ))?;
        let mut pcm_data = Vec::new();
        while let Ok(Some(packet)) = format.next_packet() {
            while !format.metadata().is_latest() {
                format.metadata().pop();
            }
            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet)? {
                // TODO: samples.resize(audio_buf.samples_interleaved(), f32::MID);
                GenericAudioBufferRef::F32(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::U8(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::U16(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::U24(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::U32(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::S8(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::S16(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::S24(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::S32(data) => conv(&mut pcm_data, data),
                GenericAudioBufferRef::F64(data) => conv(&mut pcm_data, data),
            }
        }
        Ok(AudioResampleStream {
            stream: pcm_data,
            info: AudioInfo::new(sample_rate, 1), // TODO: channel
        })
    }
}
