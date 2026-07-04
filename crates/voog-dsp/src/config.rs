//! Global DSP constants. Ported from `synth/config.py`.
//!
//! NOTE: `SAMPLE_RATE` is a compile-time constant here to keep the DSP faithful
//! to the Python reference. The engine layer will resample/adapt to the real
//! device rate if it differs.

pub const SAMPLE_RATE: f32 = 44_100.0;
pub const BUFFER_SIZE: usize = 256;
pub const MAX_VOICES: usize = 8;
pub const NUM_CHANNELS: usize = 4;
pub const NUM_OSCILLATORS: usize = 3;
pub const WAVETABLE_SIZE: usize = 2048;
/// Envelope / LFO are computed once every `CONTROL_RATE_DIVIDER` samples, then
/// linearly interpolated to audio rate.
pub const CONTROL_RATE_DIVIDER: usize = 16;
pub const A4_FREQ: f32 = 440.0;

/// MIDI note number -> frequency in Hz (equal temperament, A4 = 440).
#[inline]
pub fn midi_to_freq(note: i32) -> f32 {
    A4_FREQ * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}
