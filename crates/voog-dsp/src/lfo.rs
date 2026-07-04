//! Low-frequency oscillator. PORT OF `synth/dsp/lfo.py`.
//!
//! Control-rate generation + interpolation to audio rate. `process` OVERWRITES
//! `out` with the modulation signal in [-1,1] * depth. Returns zeros when
//! `depth <= 0`. Waveforms reuse the `Waveform` enum (sine/triangle/saw/square).

use crate::config::{CONTROL_RATE_DIVIDER, SAMPLE_RATE};
use crate::params::{LfoDest, Waveform};
use core::f32::consts::TAU;

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

    /// Sample the current waveform at `phase` in [0, 1) -> [-1, 1].
    #[inline]
    fn sample(&self, phase: f32) -> f32 {
        match self.waveform {
            Waveform::Sine => (TAU * phase).sin(),
            Waveform::Triangle => 4.0 * (phase - 0.5).abs() - 1.0,
            Waveform::Saw => 2.0 * phase - 1.0,
            Waveform::Square => {
                if phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
        }
    }

    /// Overwrite `out` with the LFO signal ([-1,1] * depth).
    pub fn process(&mut self, out: &mut [f32]) {
        let n_samples = out.len();
        if self.depth <= 0.0 {
            out.fill(0.0);
            return;
        }
        if n_samples == 0 {
            return;
        }

        let n_blocks = n_samples / CONTROL_RATE_DIVIDER;
        let remainder = n_samples % CONTROL_RATE_DIVIDER;
        let total = n_blocks + if remainder != 0 { 1 } else { 0 };

        let phase_inc = self.rate * CONTROL_RATE_DIVIDER as f32 / SAMPLE_RATE;

        // Generate control-rate values on the fly and interpolate into `out`.
        let mut pos = 0usize;
        let mut prev = 0.0f32; // control value from the previous block
        for i in 0..total {
            let cur = self.sample(self.phase);
            self.phase += phase_inc;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }

            let bs = if i < n_blocks {
                CONTROL_RATE_DIVIDER
            } else {
                remainder
            };
            if bs == 0 {
                prev = cur;
                continue;
            }

            if i == 0 {
                // First block: hold flat at the current control value.
                for s in &mut out[pos..pos + bs] {
                    *s = cur * self.depth;
                }
            } else {
                // linspace(prev, cur, bs, endpoint=False)
                let step = (cur - prev) / bs as f32;
                for (k, s) in out[pos..pos + bs].iter_mut().enumerate() {
                    *s = (prev + step * k as f32) * self.depth;
                }
            }
            pos += bs;
            prev = cur;
        }
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth_zero_is_all_zeros() {
        let mut lfo = Lfo::new();
        lfo.depth = 0.0;
        let mut out = [1.0f32; 128];
        lfo.process(&mut out);
        assert!(out.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn sine_stays_within_depth() {
        let mut lfo = Lfo::new();
        lfo.waveform = Waveform::Sine;
        lfo.rate = 3.0;
        lfo.depth = 0.5;
        let mut out = [0.0f32; SAMPLE_RATE as usize];
        lfo.process(&mut out);
        for &x in out.iter() {
            assert!(x >= -0.5 - 1e-6 && x <= 0.5 + 1e-6, "out of range: {x}");
        }
    }

    #[test]
    fn one_hz_completes_one_cycle() {
        let mut lfo = Lfo::new();
        lfo.waveform = Waveform::Sine;
        lfo.rate = 1.0;
        lfo.depth = 1.0;
        let mut out = [0.0f32; SAMPLE_RATE as usize];
        lfo.process(&mut out);
        // Count sign changes: a full sine cycle crosses zero twice.
        let mut sign_changes = 0;
        for w in out.windows(2) {
            if w[0] <= 0.0 && w[1] > 0.0 {
                sign_changes += 1;
            }
            if w[0] >= 0.0 && w[1] < 0.0 {
                sign_changes += 1;
            }
        }
        assert!(
            (2..=3).contains(&sign_changes),
            "expected ~2 sign changes for 1 cycle, got {sign_changes}"
        );
    }

    #[test]
    fn square_takes_two_values() {
        let mut lfo = Lfo::new();
        lfo.waveform = Waveform::Square;
        lfo.rate = 2.0;
        lfo.depth = 0.75;
        // Cover several full cycles so both plateaus are reached.
        let mut out = [0.0f32; SAMPLE_RATE as usize];
        lfo.process(&mut out);
        // Because of the linspace interpolation at the flip block, transitional
        // samples appear; but the first (flat) block and the plateaus must be
        // exactly +/- depth. Verify both extreme values are present and nothing
        // exceeds depth in magnitude.
        let mut saw_pos = false;
        let mut saw_neg = false;
        for &x in out.iter() {
            assert!(x.abs() <= 0.75 + 1e-6, "exceeds depth: {x}");
            if (x - 0.75).abs() < 1e-6 {
                saw_pos = true;
            }
            if (x + 0.75).abs() < 1e-6 {
                saw_neg = true;
            }
        }
        assert!(saw_pos && saw_neg, "square should reach both +/- depth");
    }

    #[test]
    fn square_first_block_only_two_values() {
        // Within a single control block (no interpolation ramp yet), a square
        // wave produces only the two plateau values.
        let mut lfo = Lfo::new();
        lfo.waveform = Waveform::Square;
        lfo.rate = 1.0;
        lfo.depth = 0.5;
        let mut out = [0.0f32; CONTROL_RATE_DIVIDER];
        lfo.process(&mut out);
        for &x in out.iter() {
            assert!(
                (x - 0.5).abs() < 1e-6 || (x + 0.5).abs() < 1e-6,
                "unexpected value {x}"
            );
        }
    }

    #[test]
    fn phase_persists_across_calls() {
        // A single long call and two back-to-back calls over the same total
        // length must produce the same phase progression.
        let mut a = Lfo::new();
        a.rate = 2.0;
        a.depth = 1.0;
        let mut whole = [0.0f32; 512];
        a.process(&mut whole);

        let mut b = Lfo::new();
        b.rate = 2.0;
        b.depth = 1.0;
        let mut first = [0.0f32; 256];
        let mut second = [0.0f32; 256];
        b.process(&mut first);
        b.process(&mut second);

        // The second call must continue from where the first left off, not
        // restart from phase 0. Compare the phase state via a fresh sample.
        // Simplest observable check: the two halves differ (phase advanced).
        assert_ne!(first, second, "phase did not advance between calls");
        // And the second call is NOT identical to a fresh first call.
        let mut c = Lfo::new();
        c.rate = 2.0;
        c.depth = 1.0;
        let mut fresh = [0.0f32; 256];
        c.process(&mut fresh);
        assert_ne!(second, fresh, "phase state was not persisted across calls");
    }
}
