//! Wavetable oscillator. PORT OF `synth/dsp/oscillator.py`.
//!
//! Implement `process_add`: phase accumulator + linear-interpolated wavetable
//! lookup, output scaled by `level` and ADDED into `out`. Honour octave,
//! semitone, detune (cents) and optional per-sample `pitch_mod` (semitones).

use crate::params::Waveform;

pub struct Oscillator {
    pub waveform: Waveform,
    pub octave: i32,
    pub semitone: i32,
    pub detune: f32, // cents
    pub level: f32,
    pub pulse_width: f32,
    phase: f32, // 0..1 accumulator
}

impl Default for Oscillator {
    fn default() -> Self {
        Self {
            waveform: Waveform::Saw,
            octave: 0,
            semitone: 0,
            detune: 0.0,
            level: 1.0,
            pulse_width: 0.5,
            phase: 0.0,
        }
    }
}

impl Oscillator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `level`-scaled oscillator output for `freq` (Hz, before oct/semi/detune)
    /// into `out`. `pitch_mod` (if present, len == out.len()) is in semitones.
    pub fn process_add(&mut self, freq: f32, out: &mut [f32], pitch_mod: Option<&[f32]>) {
        let _ = (freq, out, pitch_mod);
        todo!("port Oscillator.render from synth/dsp/oscillator.py")
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }
}
