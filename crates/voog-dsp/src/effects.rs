//! Master effects: a modulated **chorus** and a feedback **delay**.
//!
//! Both are mono, run on the mixed master signal (before the soft-clip), and
//! own their own delay lines (allocated once — no hot-path allocation). Their
//! wet amount is tied to the intensity control (chorus `depth`, delay
//! `feedback`) so that at zero they are fully transparent (bypassed).

use crate::config::SAMPLE_RATE;
use std::f32::consts::TAU;

/// Linear-interpolated read from a ring buffer, `delay` samples behind `write`.
#[inline]
fn read_delayed(buf: &[f32], write: usize, delay: f32) -> f32 {
    let len = buf.len();
    let rp = write as f32 - delay;
    let rp = if rp < 0.0 { rp + len as f32 } else { rp };
    let i0 = rp.floor() as usize % len;
    let i1 = (i0 + 1) % len;
    let frac = rp - rp.floor();
    buf[i0] * (1.0 - frac) + buf[i1] * frac
}

/// Modulated chorus (a short LFO-swept delay blended with the dry signal).
pub struct Chorus {
    pub rate: f32,  // Hz
    pub depth: f32, // 0..1 (also controls wet mix; 0 = bypassed)
    buf: Vec<f32>,
    write: usize,
    lfo_phase: f32,
}

impl Default for Chorus {
    fn default() -> Self {
        Self::new()
    }
}

impl Chorus {
    pub fn new() -> Self {
        // ~50 ms of delay line is plenty for a chorus.
        let len = (SAMPLE_RATE * 0.05) as usize + 4;
        Self {
            rate: 1.5,
            depth: 0.0,
            buf: vec![0.0; len],
            write: 0,
            lfo_phase: 0.0,
        }
    }

    /// Process the block in place.
    pub fn process(&mut self, io: &mut [f32]) {
        if self.depth <= 0.0 {
            // Bypassed — still feed the line so it doesn't click when re-enabled.
            for &x in io.iter() {
                self.buf[self.write] = x;
                self.write = (self.write + 1) % self.buf.len();
            }
            self.lfo_phase = 0.0;
            return;
        }
        let base = SAMPLE_RATE * 0.012; // 12 ms centre
        let mod_depth = SAMPLE_RATE * 0.006 * self.depth.clamp(0.0, 1.0); // up to 6 ms sweep
        let wet = self.depth.clamp(0.0, 1.0) * 0.5;
        let inc = self.rate.clamp(0.05, 12.0) / SAMPLE_RATE;
        let len = self.buf.len();
        for x in io.iter_mut() {
            let lfo = (TAU * self.lfo_phase).sin();
            self.lfo_phase += inc;
            if self.lfo_phase >= 1.0 {
                self.lfo_phase -= 1.0;
            }
            let delay = base + mod_depth * lfo;
            let delayed = read_delayed(&self.buf, self.write, delay);
            self.buf[self.write] = *x;
            self.write = (self.write + 1) % len;
            *x = *x * (1.0 - wet) + delayed * wet;
        }
    }
}

/// Feedback delay (echo). `time` sets the delay length, `feedback` the number
/// of repeats and — to keep it to two knobs — the wet amount (0 = bypassed).
pub struct Delay {
    pub time: f32,     // seconds
    pub feedback: f32, // 0..~0.9
    buf: Vec<f32>,
    write: usize,
}

impl Default for Delay {
    fn default() -> Self {
        Self::new()
    }
}

impl Delay {
    pub fn new() -> Self {
        // Up to ~1.2 s of delay.
        let len = (SAMPLE_RATE * 1.2) as usize + 4;
        Self {
            time: 0.3,
            feedback: 0.0,
            buf: vec![0.0; len],
            write: 0,
        }
    }

    /// Process the block in place.
    pub fn process(&mut self, io: &mut [f32]) {
        let fb = self.feedback.clamp(0.0, 0.92);
        if fb <= 0.0 {
            // Bypassed — decay the line so a lingering tail doesn't jump back.
            for x in io.iter() {
                self.buf[self.write] = *x;
                self.write = (self.write + 1) % self.buf.len();
            }
            return;
        }
        let max = (self.buf.len() - 2) as f32;
        let delay = (self.time.clamp(0.02, 1.2) * SAMPLE_RATE).clamp(1.0, max);
        let wet = fb * 0.6; // wet amount follows feedback so 0 = fully dry
        let len = self.buf.len();
        for x in io.iter_mut() {
            let delayed = read_delayed(&self.buf, self.write, delay);
            let dry = *x;
            self.buf[self.write] = dry + fb * delayed;
            self.write = (self.write + 1) % len;
            *x = dry + wet * delayed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chorus_bypassed_at_zero_depth() {
        let mut c = Chorus::new();
        c.depth = 0.0;
        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.01).sin()).collect();
        let mut io = input.clone();
        c.process(&mut io);
        assert_eq!(io, input, "depth=0 must be transparent");
    }

    #[test]
    fn chorus_alters_signal_when_active() {
        let mut c = Chorus::new();
        c.depth = 0.8;
        c.rate = 2.0;
        let input: Vec<f32> = (0..2048).map(|i| (i as f32 * 0.05).sin()).collect();
        let mut io = input.clone();
        c.process(&mut io);
        assert!(io.iter().all(|x| x.is_finite()));
        assert!(io != input, "active chorus should change the signal");
    }

    #[test]
    fn delay_bypassed_at_zero_feedback() {
        let mut d = Delay::new();
        d.feedback = 0.0;
        let input: Vec<f32> = (0..512).map(|i| (i as f32 * 0.02).sin()).collect();
        let mut io = input.clone();
        d.process(&mut io);
        assert_eq!(io, input, "feedback=0 must be transparent");
    }

    #[test]
    fn delay_produces_echo_and_stays_finite() {
        let mut d = Delay::new();
        d.time = 0.05;
        d.feedback = 0.6;
        // An impulse should reappear later (the echo) and never blow up.
        let mut io = vec![0.0f32; 8192];
        io[0] = 1.0;
        d.process(&mut io);
        let echo_region = &io[(0.05 * SAMPLE_RATE) as usize..];
        assert!(
            echo_region.iter().any(|&x| x.abs() > 0.05),
            "expected an audible echo"
        );
        assert!(
            io.iter().all(|x| x.is_finite() && x.abs() < 8.0),
            "delay must stay bounded"
        );
    }
}
