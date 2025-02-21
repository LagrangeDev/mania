use crate::audio::decoder::{AudioCodecDecoderError, AudioDecoder};
use crate::audio::{AudioInfo, AudioResampleStream, AudioRwStream, DecodeSample, RSStream};
use std::io::{Read, Seek, SeekFrom};
use std::marker::PhantomData;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSource, MediaSourceStream};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

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
fn conv<S, T>(samples: &mut Vec<S>, data: std::borrow::Cow<symphonia::core::audio::AudioBuffer<T>>)
where
    T: symphonia::core::sample::Sample,
    S: symphonia::core::conv::FromSample<T>,
{
    samples.extend(data.chan(0).iter().map(|v| S::from_sample(*v)));
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
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .unwrap();
        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &Default::default())
            .expect("unsupported codec");
        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(0);
        let mut pcm_data = Vec::new();
        while let Ok(packet) = format.next_packet() {
            while !format.metadata().is_latest() {
                format.metadata().pop();
            }
            if packet.track_id() != track_id {
                continue;
            }
            match decoder.decode(&packet).unwrap() {
                AudioBufferRef::F32(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U8(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U16(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U24(data) => conv(&mut pcm_data, data),
                AudioBufferRef::U32(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S8(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S16(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S24(data) => conv(&mut pcm_data, data),
                AudioBufferRef::S32(data) => conv(&mut pcm_data, data),
                AudioBufferRef::F64(data) => conv(&mut pcm_data, data),
            }
        }
        Ok(AudioResampleStream {
            stream: pcm_data,
            info: AudioInfo::new(sample_rate, 1), // TODO: channel
        })
    }
}
