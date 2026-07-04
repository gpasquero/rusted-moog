//! White / pink noise generator. PORT OF `synth/dsp/noise.py`.
//!
//! Use a small, fast, RT-safe PRNG (xorshift32) instead of numpy's RNG —
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
        Self {
            noise_type: NoiseType::White,
            level: 0.0,
            b: [0.0; 7],
            rng: 0x1234_5678,
        }
    }
}

impl NoiseGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    /// xorshift32 PRNG step. Advances `self.rng` and returns the new state.
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        // Keep the state non-zero (xorshift is degenerate at 0).
        if x == 0 {
            x = 0x1234_5678;
        }
        self.rng = x;
        x
    }

    /// Next uniform white sample in [-1.0, 1.0).
    #[inline]
    fn next_white(&mut self) -> f32 {
        // Map the top 24 bits to [0, 1) then to [-1, 1).
        let u = (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32;
        u * 2.0 - 1.0
    }

    /// Add `level`-scaled noise into `out`. No-op when `level <= 0`.
    // Coefficients are kept verbatim from the Python reference (Paul Kellet).
    #[allow(clippy::excessive_precision)]
    pub fn process_add(&mut self, out: &mut [f32]) {
        if self.level <= 0.0 {
            return;
        }
        let level = self.level;

        match self.noise_type {
            NoiseType::White => {
                for s in out.iter_mut() {
                    *s += self.next_white() * level;
                }
            }
            NoiseType::Pink => {
                let [mut b0, mut b1, mut b2, mut b3, mut b4, mut b5, mut b6] = self.b;
                for s in out.iter_mut() {
                    let w = self.next_white();
                    b0 = 0.99886 * b0 + w * 0.0555179;
                    b1 = 0.99332 * b1 + w * 0.0750759;
                    b2 = 0.96900 * b2 + w * 0.1538520;
                    b3 = 0.86650 * b3 + w * 0.3104856;
                    b4 = 0.55000 * b4 + w * 0.5329522;
                    b5 = -0.7616 * b5 - w * 0.0168980;
                    let pink = b0 + b1 + b2 + b3 + b4 + b5 + b6 + w * 0.5362;
                    b6 = w * 0.115926;
                    // 0.11 normalization, then level scaling.
                    *s += pink * 0.11 * level;
                }
                self.b = [b0, b1, b2, b3, b4, b5, b6];
            }
        }
    }

    pub fn reset(&mut self) {
        self.b = [0.0; 7];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_zero_adds_nothing() {
        let mut ng = NoiseGenerator::new();
        ng.level = 0.0;
        let mut out = [0.5f32; 64];
        ng.process_add(&mut out);
        assert!(out.iter().all(|&s| s == 0.5), "level=0 must not modify out");
    }

    #[test]
    fn white_noise_mean_near_zero_and_bounded() {
        let mut ng = NoiseGenerator::new();
        ng.noise_type = NoiseType::White;
        ng.level = 1.0;
        let mut out = vec![0.0f32; 100_000];
        ng.process_add(&mut out);

        let mut sum = 0.0f64;
        for &s in &out {
            assert!(s.is_finite(), "white sample must be finite");
            assert!(
                s.abs() <= 1.0,
                "white sample must be within [-1, 1], got {s}"
            );
            sum += s as f64;
        }
        let mean = sum / out.len() as f64;
        assert!(mean.abs() < 0.05, "white mean should be near 0, got {mean}");
    }

    #[test]
    fn pink_noise_finite_and_non_constant() {
        let mut ng = NoiseGenerator::new();
        ng.noise_type = NoiseType::Pink;
        ng.level = 1.0;
        let mut out = vec![0.0f32; 10_000];
        ng.process_add(&mut out);

        assert!(
            out.iter().all(|&s| s.is_finite()),
            "pink samples must be finite"
        );
        let first = out[0];
        assert!(
            out.iter().any(|&s| s != first),
            "pink noise must not be constant"
        );
    }

    #[test]
    fn process_add_accumulates() {
        let mut ng = NoiseGenerator::new();
        ng.noise_type = NoiseType::White;
        ng.level = 0.5;
        let mut out = [1.0f32; 256];
        ng.process_add(&mut out);
        // Each sample should be 1.0 + noise, within [1 - level, 1 + level].
        assert!(
            out.iter().any(|&s| s != 1.0),
            "process_add must add noise onto existing content"
        );
        for &s in &out {
            assert!(s.is_finite());
            assert!(
                (0.5..=1.5).contains(&s),
                "accumulated value out of range: {s}"
            );
        }
    }
}
