//! Control events applied to the [`Synth`](crate::synth::Synth) between audio
//! blocks, plus a typed parameter identifier.
//!
//! All GUI / MIDI interaction with the engine flows through `Event`s pushed on
//! a lock-free queue and drained inside the audio callback. This keeps the
//! real-time thread allocation- and lock-free.

use crate::params::{GlideMode, LfoDest, NoiseType, Patch, Waveform};

/// A continuous (f32-valued) synth parameter, addressable from GUI/MIDI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamId {
    OscOctave(usize),
    OscSemitone(usize),
    OscDetune(usize),
    OscLevel(usize),
    NoiseLevel,
    FilterCutoff,
    FilterResonance,
    FilterEnvAmount,
    FilterKeyTracking,
    AmpAttack,
    AmpDecay,
    AmpSustain,
    AmpRelease,
    FilterAttack,
    FilterDecay,
    FilterSustain,
    FilterRelease,
    LfoRate,
    LfoDepth,
    GlideTime,
}

/// An event applied to the engine. `channel` selects the multitimbral channel.
#[derive(Debug, Clone)]
pub enum Event {
    NoteOn {
        channel: u8,
        note: i32,
        velocity: u8,
    },
    NoteOff {
        channel: u8,
        note: i32,
    },
    AllNotesOff {
        channel: u8,
    },
    SetParam {
        channel: u8,
        param: ParamId,
        value: f32,
    },
    SetOscWaveform {
        channel: u8,
        osc: usize,
        waveform: Waveform,
    },
    SetNoiseType {
        channel: u8,
        noise_type: NoiseType,
    },
    SetLfoWaveform {
        channel: u8,
        waveform: Waveform,
    },
    SetLfoDest {
        channel: u8,
        dest: LfoDest,
    },
    SetGlideMode {
        channel: u8,
        mode: GlideMode,
    },
    LoadPatch {
        channel: u8,
        patch: Box<Patch>,
    },
    MasterVolume(f32),
}

impl Patch {
    /// Apply a continuous parameter change in place.
    pub fn apply_param(&mut self, param: ParamId, value: f32) {
        match param {
            ParamId::OscOctave(i) => {
                if let Some(o) = self.oscillators.get_mut(i) {
                    o.octave = value.round() as i32;
                }
            }
            ParamId::OscSemitone(i) => {
                if let Some(o) = self.oscillators.get_mut(i) {
                    o.semitone = value.round() as i32;
                }
            }
            ParamId::OscDetune(i) => {
                if let Some(o) = self.oscillators.get_mut(i) {
                    o.detune = value;
                }
            }
            ParamId::OscLevel(i) => {
                if let Some(o) = self.oscillators.get_mut(i) {
                    o.level = value;
                }
            }
            ParamId::NoiseLevel => self.noise.level = value,
            ParamId::FilterCutoff => self.filter.cutoff = value,
            ParamId::FilterResonance => self.filter.resonance = value,
            ParamId::FilterEnvAmount => self.filter.env_amount = value,
            ParamId::FilterKeyTracking => self.filter.key_tracking = value,
            ParamId::AmpAttack => self.amp_adsr.attack = value,
            ParamId::AmpDecay => self.amp_adsr.decay = value,
            ParamId::AmpSustain => self.amp_adsr.sustain = value,
            ParamId::AmpRelease => self.amp_adsr.release = value,
            ParamId::FilterAttack => self.filter_adsr.attack = value,
            ParamId::FilterDecay => self.filter_adsr.decay = value,
            ParamId::FilterSustain => self.filter_adsr.sustain = value,
            ParamId::FilterRelease => self.filter_adsr.release = value,
            ParamId::LfoRate => self.lfo.rate = value,
            ParamId::LfoDepth => self.lfo.depth = value,
            ParamId::GlideTime => self.glide.time = value,
        }
    }
}
