//! Huovilainen Moog ladder filter (24 dB/oct). PORT OF `synth/dsp/filter.py`
//! (the numba `_moog_ladder_process` reference implementation).
//!
//! Implement `process`: sample-by-sample, in place. `resonance` is 0..1 and
//! maps to feedback `r = resonance * 4.0`. Clamp cutoff to `SAMPLE_RATE*0.49`.

use crate::config::SAMPLE_RATE;
use core::f32::consts::PI;

pub struct MoogFilter {
    pub cutoff: f32,       // Hz
    pub resonance: f32,    // 0..1
    pub env_amount: f32,   // semitones (used by the voice layer)
    pub key_tracking: f32, // 0..1 (used by the voice layer)
    state: [f32; 4],
}

impl Default for MoogFilter {
    fn default() -> Self {
        Self {
            cutoff: 8000.0,
            resonance: 0.0,
            env_amount: 0.0,
            key_tracking: 0.0,
            state: [0.0; 4],
        }
    }
}

impl MoogFilter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process `io` in place. `cutoff_mod`, if present (len == io.len()), is a
    /// per-sample Hz offset ADDED to `self.cutoff` before clamping to 20..sr*0.49.
    pub fn process(&mut self, io: &mut [f32], cutoff_mod: Option<&[f32]>) {
        let sr = SAMPLE_RATE;
        let nyquist = sr * 0.49;
        let r = self.resonance * 4.0; // 0..4 range

        let [mut s0, mut s1, mut s2, mut s3] = self.state;

        for (i, sample) in io.iter_mut().enumerate() {
            // Per-sample cutoff buffer (no heap allocation).
            let fc = match cutoff_mod {
                Some(m) => (self.cutoff + m[i]).clamp(20.0, nyquist),
                None => self.cutoff.min(nyquist),
            };

            // Pre-warp (matches the numba reference branch).
            let f = if fc < nyquist {
                2.0 * sr * (PI * fc / sr).tan()
            } else {
                nyquist * 2.0
            };
            let g = f / (2.0 * sr);
            let big_g = g / (1.0 + g);

            // Feedback
            let s = big_g * big_g * big_g * s0 + big_g * big_g * s1 + big_g * s2 + s3;
            let u = (*sample - r * s) / (1.0 + r * big_g * big_g * big_g * big_g);

            // Four cascaded one-pole filters.
            let mut v = (u - s0) * big_g;
            let mut lp = v + s0;
            s0 = lp + v;
            v = (lp - s1) * big_g;
            lp = v + s1;
            s1 = lp + v;
            v = (lp - s2) * big_g;
            lp = v + s2;
            s2 = lp + v;
            v = (lp - s3) * big_g;
            lp = v + s3;
            s3 = lp + v;

            *sample = lp;
        }

        self.state = [s0, s1, s2, s3];
    }

    pub fn reset(&mut self) {
        self.state = [0.0; 4];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SAMPLE_RATE;

    /// Simple deterministic pseudo-random noise generator in [-1, 1].
    fn noise(n: usize) -> Vec<f32> {
        let mut state: u32 = 0x1234_5678;
        (0..n)
            .map(|_| {
                // xorshift32
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                (state as f32 / u32::MAX as f32) * 2.0 - 1.0
            })
            .collect()
    }

    fn rms(buf: &[f32]) -> f32 {
        (buf.iter().map(|x| x * x).sum::<f32>() / buf.len() as f32).sqrt()
    }

    #[test]
    fn stable_for_white_noise() {
        let mut f = MoogFilter::new();
        f.cutoff = 5000.0;
        f.resonance = 0.7;
        let mut io = noise(4096);
        f.process(&mut io, None);
        assert!(io.iter().all(|x| x.is_finite()), "output must be finite");
    }

    #[test]
    fn high_cutoff_passes_signal() {
        let mut f = MoogFilter::new();
        f.cutoff = SAMPLE_RATE * 0.45; // very high
        f.resonance = 0.0;
        let input = noise(4096);
        let mut io = input.clone();
        f.process(&mut io, None);
        let in_rms = rms(&input);
        let out_rms = rms(&io);
        // Roughly unchanged in level (tolerant band).
        assert!(
            out_rms > in_rms * 0.4,
            "expected pass-through, got {out_rms} vs {in_rms}"
        );
        assert!(
            out_rms < in_rms * 2.5,
            "unexpected gain, got {out_rms} vs {in_rms}"
        );
    }

    #[test]
    fn low_cutoff_attenuates_high_frequency() {
        // Feed a high-frequency tone near Nyquist; low cutoff should kill it.
        let n = 4096;
        let freq = SAMPLE_RATE * 0.4;
        let input: Vec<f32> = (0..n)
            .map(|i| (2.0 * PI * freq * i as f32 / SAMPLE_RATE).sin())
            .collect();

        let mut f = MoogFilter::new();
        f.cutoff = 200.0; // very low
        f.resonance = 0.0;
        let mut io = input.clone();
        f.process(&mut io, None);

        // Ignore the initial transient.
        let in_rms = rms(&input[n / 2..]);
        let out_rms = rms(&io[n / 2..]);
        assert!(
            out_rms < in_rms * 0.2,
            "expected strong attenuation, got {out_rms} vs {in_rms}"
        );
    }

    #[test]
    fn state_persists_across_calls() {
        let mut whole = MoogFilter::new();
        whole.cutoff = 3000.0;
        whole.resonance = 0.5;
        let input = noise(2048);
        let mut io_whole = input.clone();
        whole.process(&mut io_whole, None);

        let mut split = MoogFilter::new();
        split.cutoff = 3000.0;
        split.resonance = 0.5;
        let mut io_split = input.clone();
        let (a, b) = io_split.split_at_mut(1024);
        split.process(a, None);
        split.process(b, None);

        for (w, s) in io_whole.iter().zip(io_split.iter()) {
            assert!(
                (w - s).abs() < 1e-4,
                "split processing must match whole: {w} vs {s}"
            );
        }
    }

    #[test]
    fn resonance_increases_energy_near_cutoff() {
        let n = 8192;
        let cutoff = 1000.0;
        // Tone right at the cutoff frequency.
        let input: Vec<f32> = (0..n)
            .map(|i| (2.0 * PI * cutoff * i as f32 / SAMPLE_RATE).sin())
            .collect();

        let mut low_q = MoogFilter::new();
        low_q.cutoff = cutoff;
        low_q.resonance = 0.0;
        let mut io_low = input.clone();
        low_q.process(&mut io_low, None);

        let mut high_q = MoogFilter::new();
        high_q.cutoff = cutoff;
        high_q.resonance = 0.9;
        let mut io_high = input.clone();
        high_q.process(&mut io_high, None);

        let low_rms = rms(&io_low[n / 2..]);
        let high_rms = rms(&io_high[n / 2..]);
        assert!(
            high_rms > low_rms,
            "resonance should boost energy near cutoff: {high_rms} vs {low_rms}"
        );
    }

    #[test]
    fn cutoff_mod_is_applied() {
        let mut f = MoogFilter::new();
        f.cutoff = 500.0;
        f.resonance = 0.0;
        let input = noise(2048);

        // With a large positive modulation the filter opens up and passes more.
        let modulation = vec![10_000.0_f32; input.len()];
        let mut io_mod = input.clone();
        f.process(&mut io_mod, Some(&modulation));

        let mut f2 = MoogFilter::new();
        f2.cutoff = 500.0;
        let mut io_plain = input.clone();
        f2.process(&mut io_plain, None);

        assert!(io_mod.iter().all(|x| x.is_finite()));
        assert!(
            rms(&io_mod) > rms(&io_plain),
            "positive cutoff mod should pass more signal"
        );
    }
}
