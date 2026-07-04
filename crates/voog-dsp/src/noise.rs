//! White / pink noise generator. PORT OF `synth/dsp/noise.py`.
//!
//! Use a small, fast, RT-safe PRNG (e.g. xorshift32) instead of numpy's RNG —
//! exact sample values need not match Python, only the statistical character.
//! Pink noise uses Paul Kellet's 7-pole approximation (keep the coefficients).
//! `process_add` ADDS `level`-scaled noise into `out`.

use crate::params::NoiseType;

pub struct NoiseGenerator {
    pub noise_type: NoiseType,
    pub level: f32,
    // pink-noise state (Paul Kellet)
    b: [f32; 7],
    rng: u32,
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self { noise_type: NoiseType::White, level: 0.0, b: [0.0; 7], rng: 0x1234_5678 }
    }
}

impl NoiseGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add `level`-scaled noise into `out`. No-op when `level <= 0`.
    pub fn process_add(&mut self, out: &mut [f32]) {
        let _ = (out, &self.b, self.rng);
        todo!("port NoiseGenerator.render from synth/dsp/noise.py")
    }

    pub fn reset(&mut self) {
        self.b = [0.0; 7];
    }
}
