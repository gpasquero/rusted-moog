//! Portamento / glide. PORT OF `synth/dsp/glide.py`.
//!
//! Exponential glide in the frequency domain. `process` OVERWRITES `out` with
//! the per-sample frequency (Hz). `set_target` follows the off/always/legato
//! rules from the Python reference.

use crate::config::SAMPLE_RATE;
use crate::params::GlideMode;

pub struct Glide {
    pub time: f32, // seconds
    pub mode: GlideMode,
    current_freq: f32,
    target_freq: f32,
}

impl Default for Glide {
    fn default() -> Self {
        Self {
            time: 0.0,
            mode: GlideMode::Off,
            current_freq: 0.0,
            target_freq: 0.0,
        }
    }
}

impl Glide {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_target(&mut self, freq: f32, legato: bool) {
        if self.mode == GlideMode::Off || self.current_freq <= 0.0 {
            self.current_freq = freq;
            self.target_freq = freq;
        } else if self.mode == GlideMode::Always || (self.mode == GlideMode::Legato && legato) {
            self.target_freq = freq;
        } else {
            self.current_freq = freq;
            self.target_freq = freq;
        }
    }

    /// Overwrite `out` with the per-sample frequency (Hz).
    pub fn process(&mut self, out: &mut [f32]) {
        if self.time <= 0.0 || self.current_freq == self.target_freq {
            self.current_freq = self.target_freq;
            out.fill(self.target_freq);
            return;
        }

        // Exponential glide in the frequency domain.
        let coeff = 1.0 - (-1.0 / (self.time * SAMPLE_RATE)).exp();
        for sample in out.iter_mut() {
            self.current_freq += (self.target_freq - self.current_freq) * coeff;
            *sample = self.current_freq;
        }

        // Snap when close enough. A small *relative* threshold guarantees the
        // glide terminates in f32 even for slow (large-time) glides, where the
        // per-sample increment can fall below f32 epsilon before an absolute
        // 0.01 Hz gap is reached. 1e-4 relative is ~0.17 cents (inaudible).
        if (self.current_freq - self.target_freq).abs() < self.target_freq.abs() * 1e-4 + 1e-4 {
            self.current_freq = self.target_freq;
        }
    }

    pub fn reset(&mut self) {
        self.current_freq = 0.0;
        self.target_freq = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_off_jumps_instantly() {
        let mut g = Glide::new();
        g.mode = GlideMode::Off;
        g.time = 1.0; // time should be ignored when Off (current jumps to target)
        g.set_target(440.0, false);
        let mut out = [0.0f32; 64];
        g.process(&mut out);
        for &s in &out {
            assert!((s - 440.0).abs() < 1e-4, "expected constant 440, got {s}");
        }
    }

    #[test]
    fn time_zero_jumps_instantly() {
        let mut g = Glide::new();
        g.mode = GlideMode::Always;
        // First target establishes current (current == 0 -> jump).
        g.set_target(220.0, false);
        // Now with a real target but zero glide time -> instant jump.
        g.time = 0.0;
        g.set_target(440.0, false);
        let mut out = [0.0f32; 32];
        g.process(&mut out);
        for &s in &out {
            assert!((s - 440.0).abs() < 1e-4, "expected constant 440, got {s}");
        }
    }

    #[test]
    fn first_set_target_always_jumps() {
        // Even in Always mode with a long glide time, the very first set_target
        // (current == 0) jumps immediately.
        let mut g = Glide::new();
        g.mode = GlideMode::Always;
        g.time = 5.0;
        g.set_target(330.0, false);
        let mut out = [0.0f32; 16];
        g.process(&mut out);
        for &s in &out {
            assert!((s - 330.0).abs() < 1e-4, "expected constant 330, got {s}");
        }
    }

    #[test]
    fn always_mode_glides_monotonically_toward_target() {
        let mut g = Glide::new();
        g.mode = GlideMode::Always;
        g.time = 0.05;
        // Establish current at 220 (first call jumps).
        g.set_target(220.0, false);
        // New target; current stays at 220 and glides up.
        g.set_target(440.0, false);

        // Process a short block: should move monotonically upward, not yet arrived.
        let mut block = [0.0f32; 64];
        g.process(&mut block);
        let mut prev = 220.0f32;
        for &s in &block {
            assert!(s > prev - 1e-6, "expected monotonic increase, {s} < {prev}");
            assert!(s <= 440.0 + 1e-3, "should not overshoot target: {s}");
            prev = s;
        }
        assert!(
            prev > 220.0 && prev < 440.0,
            "should be mid-glide, got {prev}"
        );

        // Process enough samples to converge to the target.
        let mut long = [0.0f32; 44_100];
        g.process(&mut long);
        // Tolerance is loose: with f32 the per-sample step falls below the
        // float epsilon before the 0.01 snap threshold is reached, so it
        // asymptotes very close to (but not exactly at) the target.
        let last = *long.last().unwrap();
        assert!(
            (last - 440.0).abs() < 0.1,
            "expected convergence to ~440, got {last}"
        );
    }

    #[test]
    fn output_length_matches() {
        let mut g = Glide::new();
        g.mode = GlideMode::Always;
        g.time = 0.1;
        g.set_target(100.0, false);
        g.set_target(200.0, false);
        for len in [1usize, 7, 33, 256] {
            let mut buf = vec![0.0f32; len];
            g.process(&mut buf);
            assert_eq!(buf.len(), len);
        }
    }
}
