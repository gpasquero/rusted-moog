//! Portamento / glide. PORT OF `synth/dsp/glide.py`.
//!
//! Exponential glide in the frequency domain. `process` OVERWRITES `out` with
//! the per-sample frequency (Hz). `set_target` follows the off/always/legato
//! rules from the Python reference.

use crate::params::GlideMode;

pub struct Glide {
    pub time: f32, // seconds
    pub mode: GlideMode,
    current_freq: f32,
    target_freq: f32,
}

impl Default for Glide {
    fn default() -> Self {
        Self { time: 0.0, mode: GlideMode::Off, current_freq: 0.0, target_freq: 0.0 }
    }
}

impl Glide {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, freq: f32, legato: bool) {
        let _ = (freq, legato);
        todo!("port Glide.set_target from synth/dsp/glide.py")
    }

    /// Overwrite `out` with the per-sample frequency (Hz).
    pub fn process(&mut self, out: &mut [f32]) {
        let _ = (out, self.current_freq, self.target_freq);
        todo!("port Glide.render from synth/dsp/glide.py")
    }

    pub fn reset(&mut self) {
        self.current_freq = 0.0;
        self.target_freq = 0.0;
    }
}
