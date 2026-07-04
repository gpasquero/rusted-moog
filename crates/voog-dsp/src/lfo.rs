//! Low-frequency oscillator. PORT OF `synth/dsp/lfo.py`.
//!
//! Control-rate generation + interpolation to audio rate. `process` OVERWRITES
//! `out` with the modulation signal in [-1,1] * depth. Returns zeros when
//! `depth <= 0`. Waveforms reuse the `Waveform` enum (sine/triangle/saw/square).

use crate::params::{LfoDest, Waveform};

pub struct Lfo {
    pub waveform: Waveform,
    pub rate: f32,  // Hz
    pub depth: f32, // 0..1
    pub destination: LfoDest,
    pub key_sync: bool,
    phase: f32,
}

impl Default for Lfo {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            rate: 1.0,
            depth: 0.0,
            destination: LfoDest::Filter,
            key_sync: true,
            phase: 0.0,
        }
    }
}

impl Lfo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Overwrite `out` with the LFO signal ([-1,1] * depth).
    pub fn process(&mut self, out: &mut [f32]) {
        let _ = (out, self.phase);
        todo!("port LFO.render from synth/dsp/lfo.py")
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}
