//! A single polyphonic voice. PORT OF `synth/engine/voice.py`.
//!
//! `render_add` mixes 3 oscillators (+ noise) through the Moog filter and the
//! amp VCA, with dual ADSR envelopes, LFO modulation and glide. All work uses
//! pre-allocated scratch buffers so the audio thread never allocates.

use crate::config::{midi_to_freq, A4_FREQ, BUFFER_SIZE, NUM_OSCILLATORS};
use crate::envelope::Adsr;
use crate::filter::MoogFilter;
use crate::glide::Glide;
use crate::lfo::Lfo;
use crate::noise::NoiseGenerator;
use crate::oscillator::Oscillator;
use crate::params::{LfoDest, Patch};

/// Scratch buffers reused every block (length `BUFFER_SIZE`).
struct Scratch {
    amp_env: [f32; BUFFER_SIZE],
    filt_env: [f32; BUFFER_SIZE],
    lfo: [f32; BUFFER_SIZE],
    pitch_mod: [f32; BUFFER_SIZE],
    freq: [f32; BUFFER_SIZE],
    mix: [f32; BUFFER_SIZE],
    cutoff_mod: [f32; BUFFER_SIZE],
}

impl Scratch {
    fn new() -> Self {
        Self {
            amp_env: [0.0; BUFFER_SIZE],
            filt_env: [0.0; BUFFER_SIZE],
            lfo: [0.0; BUFFER_SIZE],
            pitch_mod: [0.0; BUFFER_SIZE],
            freq: [0.0; BUFFER_SIZE],
            mix: [0.0; BUFFER_SIZE],
            cutoff_mod: [0.0; BUFFER_SIZE],
        }
    }
}

pub struct Voice {
    oscillators: [Oscillator; NUM_OSCILLATORS],
    noise: NoiseGenerator,
    filter: MoogFilter,
    amp_env: Adsr,
    filter_env: Adsr,
    lfo: Lfo,
    glide: Glide,

    pub note: i32,
    velocity: f32,
    pub active: bool,
    base_freq: f32,

    scratch: Scratch,
}

impl Default for Voice {
    fn default() -> Self {
        Self::new()
    }
}

impl Voice {
    pub fn new() -> Self {
        Self {
            oscillators: [Oscillator::new(), Oscillator::new(), Oscillator::new()],
            noise: NoiseGenerator::new(),
            filter: MoogFilter::new(),
            amp_env: Adsr::new(),
            filter_env: Adsr::new(),
            lfo: Lfo::new(),
            glide: Glide::new(),
            note: -1,
            velocity: 0.0,
            active: false,
            base_freq: 0.0,
            scratch: Scratch::new(),
        }
    }

    /// Copy all patch parameters onto this voice's DSP units.
    pub fn apply_patch(&mut self, patch: &Patch) {
        for (i, osc) in self.oscillators.iter_mut().enumerate() {
            if let Some(op) = patch.oscillators.get(i) {
                osc.waveform = op.waveform;
                osc.octave = op.octave;
                osc.semitone = op.semitone;
                osc.detune = op.detune;
                osc.level = op.level;
                osc.pulse_width = op.pulse_width;
            }
        }
        self.noise.noise_type = patch.noise.noise_type;
        self.noise.level = patch.noise.level;
        self.filter.cutoff = patch.filter.cutoff;
        self.filter.resonance = patch.filter.resonance;
        self.filter.env_amount = patch.filter.env_amount;
        self.filter.key_tracking = patch.filter.key_tracking;
        self.amp_env.attack = patch.amp_adsr.attack;
        self.amp_env.decay = patch.amp_adsr.decay;
        self.amp_env.sustain = patch.amp_adsr.sustain;
        self.amp_env.release = patch.amp_adsr.release;
        self.filter_env.attack = patch.filter_adsr.attack;
        self.filter_env.decay = patch.filter_adsr.decay;
        self.filter_env.sustain = patch.filter_adsr.sustain;
        self.filter_env.release = patch.filter_adsr.release;
        self.lfo.waveform = patch.lfo.waveform;
        self.lfo.rate = patch.lfo.rate;
        self.lfo.depth = patch.lfo.depth;
        self.lfo.destination = patch.lfo.destination;
        self.lfo.key_sync = patch.lfo.key_sync;
        self.glide.time = patch.glide.time;
        self.glide.mode = patch.glide.mode;
    }

    pub fn note_on(&mut self, note: i32, velocity: u8, legato: bool) {
        self.note = note;
        self.velocity = velocity as f32 / 127.0;
        self.base_freq = midi_to_freq(note);
        self.active = true;
        self.glide.set_target(self.base_freq, legato);
        self.amp_env.gate_on();
        self.filter_env.gate_on();
        if self.lfo.key_sync {
            self.lfo.reset();
        }
        if !legato {
            for osc in &mut self.oscillators {
                osc.reset_phase();
            }
        }
    }

    pub fn note_off(&mut self) {
        self.amp_env.gate_off();
        self.filter_env.gate_off();
    }

    /// Render this voice and ADD it into `out` (len must be <= BUFFER_SIZE).
    pub fn render_add(&mut self, out: &mut [f32]) {
        if !self.active {
            return;
        }
        let n = out.len();
        debug_assert!(n <= BUFFER_SIZE);

        // Amp envelope. Deactivate if the envelope has fully released.
        let amp_env = &mut self.scratch.amp_env[..n];
        self.amp_env.process(amp_env);
        if !self.amp_env.is_active() && amp_env.iter().copied().fold(0.0f32, f32::max) < 1e-5 {
            self.active = false;
            return;
        }

        // Filter envelope + LFO.
        let filt_env = &mut self.scratch.filt_env[..n];
        self.filter_env.process(filt_env);
        let lfo = &mut self.scratch.lfo[..n];
        self.lfo.process(lfo);

        // Pitch modulation from the LFO (semitones), if routed to pitch.
        let pitch_is_lfo = self.lfo.destination == LfoDest::Pitch && self.lfo.depth > 0.0;
        let pitch_mod = &mut self.scratch.pitch_mod[..n];
        if pitch_is_lfo {
            for (p, &l) in pitch_mod.iter_mut().zip(lfo.iter()) {
                *p = l * 12.0;
            }
        }

        // Glide -> per-sample frequency; oscillators use the block mean (glide
        // is slow-moving), matching the Python reference (mean hoisted out of
        // the oscillator loop — the Python recomputed it per oscillator).
        let freq = &mut self.scratch.freq[..n];
        self.glide.process(freq);
        let mean_freq = freq.iter().sum::<f32>() / n as f32;

        // Mix oscillators.
        let mix = &mut self.scratch.mix[..n];
        mix.fill(0.0);
        for osc in &mut self.oscillators {
            if osc.level > 0.0 {
                let pm = if pitch_is_lfo {
                    Some(&pitch_mod[..n])
                } else {
                    None
                };
                osc.process_add(mean_freq, mix, pm);
            }
        }
        // Noise.
        if self.noise.level > 0.0 {
            self.noise.process_add(mix);
        }

        // Filter cutoff modulation. NOTE: faithful to the Python — `key_tracking`
        // only scales the modulation depth (via `base_cutoff`); the filter's own
        // `cutoff` is the modulation baseline inside `MoogFilter::process`.
        let mut base_cutoff = self.filter.cutoff;
        if self.filter.key_tracking > 0.0 {
            base_cutoff += (self.base_freq - A4_FREQ) * self.filter.key_tracking;
        }
        let cutoff_mod = &mut self.scratch.cutoff_mod[..n];
        cutoff_mod.fill(0.0);
        let mut any_mod = false;
        if self.filter.env_amount != 0.0 {
            let amt = self.filter.env_amount;
            for (c, &e) in cutoff_mod.iter_mut().zip(filt_env.iter()) {
                *c += base_cutoff * (2.0f32.powf(e * amt / 12.0) - 1.0);
            }
            any_mod = true;
        }
        if self.lfo.destination == LfoDest::Filter && self.lfo.depth > 0.0 {
            for (c, &l) in cutoff_mod.iter_mut().zip(lfo.iter()) {
                *c += base_cutoff * (2.0f32.powf(l * 2.0 / 12.0) - 1.0);
            }
            any_mod = true;
        }

        // Apply the filter (in place on `mix`).
        if any_mod {
            self.filter.process(mix, Some(cutoff_mod));
        } else {
            self.filter.process(mix, None);
        }

        // Amp tremolo from the LFO.
        if self.lfo.destination == LfoDest::Amp && self.lfo.depth > 0.0 {
            for (m, &l) in mix.iter_mut().zip(lfo.iter()) {
                *m *= 1.0 + l * 0.5;
            }
        }

        // Amp envelope + velocity, summed into the output.
        let vel = self.velocity;
        for ((o, &m), &a) in out.iter_mut().zip(mix.iter()).zip(amp_env.iter()) {
            *o += m * a * vel;
        }
    }

    pub fn reset(&mut self) {
        self.note = -1;
        self.velocity = 0.0;
        self.active = false;
        self.amp_env.reset();
        self.filter_env.reset();
        self.lfo.reset();
        self.glide.reset();
        self.filter.reset();
        self.noise.reset();
        for osc in &mut self.oscillators {
            osc.reset_phase();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn saw_patch() -> Patch {
        Patch::default()
    }

    #[test]
    fn inactive_voice_adds_nothing() {
        let mut v = Voice::new();
        let mut out = [1.0f32; 128];
        v.render_add(&mut out);
        assert!(out.iter().all(|&s| s == 1.0));
    }

    #[test]
    fn note_on_produces_sound() {
        let mut v = Voice::new();
        v.apply_patch(&saw_patch());
        v.note_on(69, 100, false); // A4
        let mut out = [0.0f32; 256];
        v.render_add(&mut out);
        assert!(v.active);
        assert!(
            out.iter().any(|&s| s.abs() > 1e-4),
            "voice should produce audio"
        );
        assert!(out.iter().all(|&s| s.is_finite()));
    }

    #[test]
    fn note_off_eventually_silences_and_deactivates() {
        let mut p = saw_patch();
        p.amp_adsr.release = 0.01;
        let mut v = Voice::new();
        v.apply_patch(&p);
        v.note_on(60, 127, false);
        let mut out = [0.0f32; 256];
        v.render_add(&mut out);
        v.note_off();
        // Render enough blocks for the short release to finish.
        for _ in 0..50 {
            let mut b = [0.0f32; 256];
            v.render_add(&mut b);
            if !v.active {
                break;
            }
        }
        assert!(!v.active, "voice should deactivate after release");
    }

    #[test]
    fn output_stays_bounded() {
        let mut v = Voice::new();
        v.apply_patch(&saw_patch());
        v.note_on(72, 127, false);
        for _ in 0..20 {
            let mut out = [0.0f32; 256];
            v.render_add(&mut out);
            for &s in &out {
                assert!(s.is_finite() && s.abs() < 8.0, "unbounded sample {s}");
            }
        }
    }
}
