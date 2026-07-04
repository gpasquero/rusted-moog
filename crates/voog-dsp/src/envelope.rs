//! ADSR envelope. PORT OF `synth/dsp/envelope.py`.
//!
//! Compute at control rate (blocks of `CONTROL_RATE_DIVIDER`) then linearly
//! interpolate to audio rate, exactly like the Python `render`. `process`
//! OVERWRITES `out` with the per-sample envelope (0..1).

use crate::config::{BUFFER_SIZE, CONTROL_RATE_DIVIDER, SAMPLE_RATE};

/// Minimum time to avoid division by zero (Python `_MIN_TIME`).
const MIN_TIME: f32 = 0.001;

/// Maximum number of control-rate blocks in a single `process` call. A block is
/// `CONTROL_RATE_DIVIDER` samples plus one extra for a partial remainder block.
/// Sized generously so callers using buffers up to `BUFFER_SIZE` never overflow.
const MAX_BLOCKS: usize = BUFFER_SIZE / CONTROL_RATE_DIVIDER + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Adsr {
    pub attack: f32,  // seconds
    pub decay: f32,   // seconds
    pub sustain: f32, // 0..1
    pub release: f32, // seconds
    state: EnvState,
    level: f32,
    samples_in_state: usize,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
            state: EnvState::Idle,
            level: 0.0,
            samples_in_state: 0,
        }
    }
}

impl Adsr {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gate_on(&mut self) {
        self.state = EnvState::Attack;
        self.samples_in_state = 0;
    }

    pub fn gate_off(&mut self) {
        if self.state != EnvState::Idle {
            self.state = EnvState::Release;
            self.samples_in_state = 0;
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvState::Idle
    }

    /// Advance the envelope at control rate by `n_samples`, updating `level`
    /// and transitioning state. Mirrors Python `_advance`.
    fn advance(&mut self, n_samples: usize) {
        self.samples_in_state += n_samples;
        let n = n_samples as f32;
        match self.state {
            EnvState::Attack => {
                let rate = self.attack.max(MIN_TIME) * SAMPLE_RATE;
                self.level += n / rate;
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = EnvState::Decay;
                    self.samples_in_state = 0;
                }
            }
            EnvState::Decay => {
                let rate = self.decay.max(MIN_TIME) * SAMPLE_RATE;
                self.level -= (1.0 - self.sustain) * n / rate;
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.state = EnvState::Sustain;
                    self.samples_in_state = 0;
                }
            }
            EnvState::Sustain => {
                self.level = self.sustain;
            }
            EnvState::Release => {
                let rate = self.release.max(MIN_TIME) * SAMPLE_RATE;
                self.level -= self.level * n / rate;
                if self.level < 1e-5 {
                    self.level = 0.0;
                    self.state = EnvState::Idle;
                    self.samples_in_state = 0;
                }
            }
            EnvState::Idle => {}
        }
    }

    /// Overwrite `out` with the per-sample envelope value.
    ///
    /// Renders at control rate (blocks of `CONTROL_RATE_DIVIDER`), then linearly
    /// interpolates the control values to audio rate: block 0 is flat at its
    /// value, subsequent blocks ramp from the previous to the current value.
    pub fn process(&mut self, out: &mut [f32]) {
        let n_samples = out.len();
        if n_samples == 0 {
            return;
        }

        let n_blocks = n_samples / CONTROL_RATE_DIVIDER;
        let remainder = n_samples % CONTROL_RATE_DIVIDER;
        let total_blocks = n_blocks + usize::from(remainder != 0);

        assert!(
            total_blocks <= MAX_BLOCKS,
            "envelope: out.len() = {n_samples} exceeds supported max of {}",
            MAX_BLOCKS * CONTROL_RATE_DIVIDER
        );

        // Control-rate values, one per block.
        let mut control_values = [0.0f32; MAX_BLOCKS];
        for (i, cv) in control_values.iter_mut().enumerate().take(total_blocks) {
            let block_size = if i < n_blocks {
                CONTROL_RATE_DIVIDER
            } else {
                remainder
            };
            self.advance(block_size);
            *cv = self.level;
        }

        // If a single (or partial) block, fill flat with the current level.
        if total_blocks <= 1 {
            out.fill(self.level);
            return;
        }

        // Interpolate control values to audio rate.
        let mut pos = 0usize;
        for i in 0..total_blocks {
            let block_size = if i < n_blocks {
                CONTROL_RATE_DIVIDER
            } else {
                remainder
            };
            if block_size == 0 {
                continue;
            }
            if i == 0 {
                out[pos..pos + block_size].fill(control_values[i]);
            } else {
                let prev = control_values[i - 1];
                let cur = control_values[i];
                let inv = 1.0 / block_size as f32;
                for (j, s) in out[pos..pos + block_size].iter_mut().enumerate() {
                    let t = j as f32 * inv;
                    *s = prev + (cur - prev) * t;
                }
            }
            pos += block_size;
        }
    }

    pub fn reset(&mut self) {
        self.state = EnvState::Idle;
        self.level = 0.0;
        self.samples_in_state = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_in_unit_range(buf: &[f32]) {
        for &s in buf {
            assert!(
                (0.0..=1.0).contains(&s),
                "sample {s} out of [0,1] range"
            );
        }
    }

    #[test]
    fn attack_rises_then_decays_to_sustain() {
        let mut env = Adsr::new();
        env.attack = 0.02;
        env.decay = 0.05;
        env.sustain = 0.6;
        env.release = 0.1;
        env.gate_on();

        let mut buf = [0.0f32; 64];

        // First block: should be rising during attack, above zero.
        env.process(&mut buf);
        all_in_unit_range(&buf);
        assert!(buf[buf.len() - 1] > buf[0], "envelope should rise in attack");

        // Keep processing until we reach the peak (~1.0).
        let mut peak = 0.0f32;
        for _ in 0..200 {
            env.process(&mut buf);
            all_in_unit_range(&buf);
            for &s in &buf {
                peak = peak.max(s);
            }
        }
        assert!(peak > 0.99, "attack should reach ~1.0, got {peak}");

        // Eventually settle to sustain.
        for _ in 0..400 {
            env.process(&mut buf);
            all_in_unit_range(&buf);
        }
        let last = buf[buf.len() - 1];
        assert!(
            (last - env.sustain).abs() < 0.02,
            "should hold near sustain {}, got {last}",
            env.sustain
        );
    }

    #[test]
    fn sustain_holds() {
        let mut env = Adsr::new();
        env.attack = 0.001;
        env.decay = 0.001;
        env.sustain = 0.5;
        env.gate_on();

        let mut buf = [0.0f32; 64];
        for _ in 0..100 {
            env.process(&mut buf);
        }
        // Now firmly in sustain.
        for _ in 0..10 {
            env.process(&mut buf);
            all_in_unit_range(&buf);
            for &s in &buf {
                assert!((s - 0.5).abs() < 1e-4, "sustain should hold at 0.5, got {s}");
            }
        }
        assert_eq!(env.state, EnvState::Sustain);
    }

    #[test]
    fn release_decays_to_zero_and_inactive() {
        let mut env = Adsr::new();
        env.attack = 0.001;
        env.decay = 0.001;
        env.sustain = 0.7;
        env.release = 0.02;
        env.gate_on();

        let mut buf = [0.0f32; 64];
        for _ in 0..100 {
            env.process(&mut buf);
        }
        assert!(env.is_active());

        env.gate_off();
        // Release should drive level down and eventually go idle.
        let mut became_idle = false;
        for _ in 0..2000 {
            env.process(&mut buf);
            all_in_unit_range(&buf);
            if !env.is_active() {
                became_idle = true;
                break;
            }
        }
        assert!(became_idle, "envelope should become inactive after release");
        // Once idle, output is flat zero.
        env.process(&mut buf);
        for &s in &buf {
            assert_eq!(s, 0.0);
        }
    }

    #[test]
    fn short_attack_reaches_near_one_quickly() {
        let mut env = Adsr::new();
        env.attack = 0.001; // MIN_TIME
        env.decay = 10.0; // long decay so we can observe the peak
        env.sustain = 0.0;
        env.gate_on();

        // 0.001s * 44100 ~= 44 samples to reach 1.0, well within one buffer.
        let mut buf = [0.0f32; 64];
        env.process(&mut buf);
        all_in_unit_range(&buf);
        let peak = buf.iter().cloned().fold(0.0f32, f32::max);
        assert!(peak > 0.99, "short attack should reach ~1.0 fast, got {peak}");
    }

    #[test]
    fn output_always_in_unit_range_various_sizes() {
        let mut env = Adsr::new();
        env.gate_on();
        for &len in &[1usize, 7, 16, 17, 63, 64] {
            let mut buf = vec![0.0f32; len];
            env.process(&mut buf);
            all_in_unit_range(&buf);
        }
    }
}
