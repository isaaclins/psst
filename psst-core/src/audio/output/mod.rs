use crate::audio::source::AudioSource;

#[cfg(feature = "cpal")]
pub mod cpal;
#[cfg(feature = "cubeb")]
pub mod cubeb;

#[cfg(all(feature = "cubeb", feature = "cpal"))]
compile_error!("features `cubeb` and `cpal` are mutually exclusive; enable only one audio backend");

#[cfg(not(any(feature = "cubeb", feature = "cpal")))]
compile_error!("enable either the `cpal` or `cubeb` feature to build audio output support");

#[cfg(all(feature = "cubeb", not(feature = "cpal")))]
pub type DefaultAudioOutput = cubeb::CubebOutput;
#[cfg(all(feature = "cpal", not(feature = "cubeb")))]
pub type DefaultAudioOutput = cpal::CpalOutput;

pub type DefaultAudioSink = <DefaultAudioOutput as AudioOutput>::Sink;

pub trait AudioOutput {
    type Sink: AudioSink;

    fn sink(&self) -> Self::Sink;
}

pub trait AudioSink {
    fn channel_count(&self) -> usize;
    fn sample_rate(&self) -> u32;
    fn set_volume(&self, volume: f32);
    fn play(&self, source: impl AudioSource);
    fn pause(&self);
    fn resume(&self);
    fn stop(&self);
    fn close(&self);
}
