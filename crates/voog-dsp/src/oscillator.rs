//! Wavetable oscillator. PORT OF `synth/dsp/oscillator.py`.
//!
//! Implement `process_add`: phase accumulator + linear-interpolated wavetable
//! lookup, output scaled by `level` and ADDED into `out`. Honour octave,
//! semitone, detune (cents) and optional per-sample `pitch_mod` (semitones).

use crate::config::{SAMPLE_RATE, WAVETABLE_SIZE};
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
        // Apply octave, semitone, detune (cents).
        let f = freq
            * 2.0f32.powi(self.octave)
            * 2.0f32.powf(self.semitone as f32 / 12.0)
            * 2.0f32.powf(self.detune / 1200.0);

        if self.level <= 0.0 {
            return;
        }

        let table = crate::wavetables::wavetable(self.waveform);
        let ts = WAVETABLE_SIZE as f32;

        let base_inc = f / SAMPLE_RATE;
        let mut phase = self.phase;

        for (i, sample) in out.iter_mut().enumerate() {
            // Per-sample phase increment.
            let inc = match pitch_mod {
                Some(pm) => (f * 2.0f32.powf(pm[i] / 12.0)) / SAMPLE_RATE,
                None => base_inc,
            };

            // Linear-interpolated wavetable lookup at the current phase
            // (matches Python: phase is read BEFORE adding the increment).
            let idx_f = phase * ts;
            let idx_i = idx_f as usize % WAVETABLE_SIZE;
            let frac = idx_f - (idx_f as usize) as f32;
            let idx_next = (idx_i + 1) % WAVETABLE_SIZE;
            let value = table[idx_i] * (1.0 - frac) + table[idx_next] * frac;

            *sample += value * self.level;

            // Advance and wrap the phase accumulator.
            phase += inc;
            phase -= phase.floor();
        }

        self.phase = phase;
    }

    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SAMPLE_RATE;

    #[test]
    fn output_within_bounds() {
        for wf in [
            Waveform::Sine,
            Waveform::Saw,
            Waveform::Square,
            Waveform::Triangle,
        ] {
            let mut osc = Oscillator {
                waveform: wf,
                ..Default::default()
            };
            let mut out = vec![0.0f32; 4096];
            osc.process_add(220.0, &mut out, None);
            for &s in &out {
                assert!(
                    (-1.5..=1.5).contains(&s),
                    "sample {s} out of range for {wf:?}"
                );
            }
        }
    }

    #[test]
    fn processes_all_samples() {
        // Every sample of a fresh (zeroed) buffer should be written for a
        // nonzero-level oscillator.
        let mut osc = Oscillator {
            waveform: Waveform::Saw,
            ..Default::default()
        };
        let mut out = vec![0.0f32; 512];
        osc.process_add(440.0, &mut out, None);
        // Saw is nonzero almost everywhere; at least require the phase advanced
        // and the buffer is not all zeros.
        assert!(out.iter().any(|&s| s != 0.0));
        assert_eq!(out.len(), 512);
    }

    #[test]
    fn sine_matches_table_shape() {
        // A low frequency sine should closely track an analytic sine at the
        // accumulated phase. Start from phase 0.
        let freq = 55.0f32;
        let mut osc = Oscillator {
            waveform: Waveform::Sine,
            ..Default::default()
        };
        let n = 2048;
        let mut out = vec![0.0f32; n];
        osc.process_add(freq, &mut out, None);

        let inc = freq / SAMPLE_RATE;
        for (i, &s) in out.iter().enumerate() {
            let phase = (i as f32) * inc; // phase read before increment each step
            let expected = (2.0 * std::f32::consts::PI * phase).sin();
            assert!(
                (s - expected).abs() < 1e-2,
                "sine mismatch at {i}: got {s}, expected {expected}"
            );
        }
    }

    #[test]
    fn level_zero_adds_nothing() {
        let mut osc = Oscillator {
            waveform: Waveform::Saw,
            level: 0.0,
            ..Default::default()
        };
        let mut out = vec![0.0f32; 256];
        osc.process_add(440.0, &mut out, None);
        assert!(out.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn phase_continuity_across_calls() {
        // Rendering N+N samples in two calls must equal rendering 2N in one
        // call: the phase persists with no discontinuity at the boundary.
        let freq = 330.0f32;
        let n = 512;

        let mut osc_single = Oscillator {
            waveform: Waveform::Saw,
            ..Default::default()
        };
        let mut single = vec![0.0f32; 2 * n];
        osc_single.process_add(freq, &mut single, None);

        let mut osc_split = Oscillator {
            waveform: Waveform::Saw,
            ..Default::default()
        };
        let mut a = vec![0.0f32; n];
        let mut b = vec![0.0f32; n];
        osc_split.process_add(freq, &mut a, None);
        osc_split.process_add(freq, &mut b, None);

        for i in 0..n {
            assert!((single[i] - a[i]).abs() < 1e-6, "first half mismatch at {i}");
            assert!(
                (single[n + i] - b[i]).abs() < 1e-6,
                "second half mismatch at {i}"
            );
        }
    }

    #[test]
    fn pitch_mod_raises_pitch() {
        // +12 semitones should double the effective frequency -> phase advances
        // twice as fast.
        let freq = 110.0f32;
        let n = 1024;

        let mut osc_mod = Oscillator {
            waveform: Waveform::Sine,
            ..Default::default()
        };
        let pm = vec![12.0f32; n];
        let mut modded = vec![0.0f32; n];
        osc_mod.process_add(freq, &mut modded, Some(&pm));

        let mut osc_ref = Oscillator {
            waveform: Waveform::Sine,
            ..Default::default()
        };
        let mut reference = vec![0.0f32; n];
        osc_ref.process_add(freq * 2.0, &mut reference, None);

        for i in 0..n {
            assert!(
                (modded[i] - reference[i]).abs() < 1e-5,
                "pitch_mod mismatch at {i}"
            );
        }
    }

    #[test]
    fn adds_into_existing_buffer() {
        let mut osc = Oscillator {
            waveform: Waveform::Saw,
            ..Default::default()
        };
        let mut out = vec![1.0f32; 64];
        let mut fresh = vec![0.0f32; 64];
        let mut osc2 = Oscillator {
            waveform: Waveform::Saw,
            ..Default::default()
        };
        osc.process_add(440.0, &mut out, None);
        osc2.process_add(440.0, &mut fresh, None);
        for i in 0..64 {
            assert!((out[i] - (1.0 + fresh[i])).abs() < 1e-6);
        }
    }
}
