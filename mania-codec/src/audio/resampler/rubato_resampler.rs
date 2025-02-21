use crate::audio::resampler::{AudioCodecResamplerError, AudioResampler};
use crate::audio::{AudioInfo, AudioResampleStream, ResampleSample};
use num_traits::FromPrimitive;
use rubato::Resampler;
use std::marker::PhantomData;

pub struct RubatoResampler<U: ResampleSample> {
    target_sample_rate: u32,
    _phantom: PhantomData<U>,
}

impl<U: ResampleSample> RubatoResampler<U> {
    pub fn new(target_sample_rate: u32) -> Self {
        Self {
            target_sample_rate,
            _phantom: PhantomData,
        }
    }
}

// ref: https://github.com/kyutai-labs/hibiki/blob/main/hibiki-rs/src/audio_io.rs
impl AudioResampler<f32, i16> for RubatoResampler<i16> {
    fn resample(
        &self,
        input: &AudioResampleStream<f32>,
    ) -> Result<AudioResampleStream<i16>, AudioCodecResamplerError> {
        let (stream, info) = (&input.stream, &input.info);
        let pcm_in = stream;
        let sr_in = info.sample_rate as usize;
        let sr_out = self.target_sample_rate as usize;
        let mut pcm_out = Vec::with_capacity(
            (pcm_in.len() as f64 * sr_out as f64 / sr_in as f64) as usize + 1024,
        );
        // TODO: channel
        let mut resampler = rubato::FftFixedInOut::<f32>::new(sr_in, sr_out, 1024, 1)?;
        let mut output_buffer = resampler.output_buffer_allocate(true);
        let mut pos_in = 0;

        while pos_in + resampler.input_frames_next() < pcm_in.len() {
            let (in_len, out_len) =
                resampler.process_into_buffer(&[&pcm_in[pos_in..]], &mut output_buffer, None)?;
            pos_in += in_len;
            pcm_out.extend_from_slice(&output_buffer[0][..out_len]);
        }

        if pos_in < pcm_in.len() {
            let (_, out_len) = resampler.process_partial_into_buffer(
                Some(&[&pcm_in[pos_in..]]),
                &mut output_buffer,
                None,
            )?;
            pcm_out.extend_from_slice(&output_buffer[0][..out_len]);
        }

        let data = pcm_out
            .iter()
            .filter_map(|&f32_value| i16::from_f32(f32_value * f32::from_i16(i16::MAX).unwrap()))
            .collect::<Vec<_>>();

        Ok(AudioResampleStream {
            stream: data,
            info: AudioInfo::new(self.target_sample_rate, 1), // TODO: dynamic channel
        })
    }
}
