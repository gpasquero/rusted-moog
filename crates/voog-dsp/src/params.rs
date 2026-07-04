//! Patch / parameter data model. Ported from `synth/patch/patch.py`.
//!
//! `serde` derives use `rename_all = "lowercase"` so enums (de)serialize as the
//! same lowercase strings the Python version stored in JSON patches.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Waveform {
    Sine,
    Saw,
    Square,
    Triangle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoiseType {
    White,
    Pink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LfoDest {
    Filter,
    Pitch,
    Amp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GlideMode {
    Off,
    Always,
    Legato,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OscParams {
    pub waveform: Waveform,
    pub octave: i32,   // -2..+2
    pub semitone: i32, // -12..+12
    pub detune: f32,   // cents
    pub level: f32,
    pub pulse_width: f32,
}

impl Default for OscParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Saw,
            octave: 0,
            semitone: 0,
            detune: 0.0,
            level: 1.0,
            pulse_width: 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NoiseParams {
    pub noise_type: NoiseType,
    pub level: f32,
}

impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            noise_type: NoiseType::White,
            level: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FilterParams {
    pub cutoff: f32,       // Hz
    pub resonance: f32,    // 0..1
    pub env_amount: f32,   // semitones
    pub key_tracking: f32, // 0..1
}

impl Default for FilterParams {
    fn default() -> Self {
        Self {
            cutoff: 8000.0,
            resonance: 0.0,
            env_amount: 0.0,
            key_tracking: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AdsrParams {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for AdsrParams {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LfoParams {
    pub waveform: Waveform,
    pub rate: f32,
    pub depth: f32,
    pub destination: LfoDest,
    pub key_sync: bool,
}

impl Default for LfoParams {
    fn default() -> Self {
        Self {
            waveform: Waveform::Sine,
            rate: 1.0,
            depth: 0.0,
            destination: LfoDest::Filter,
            key_sync: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlideParams {
    pub time: f32,
    pub mode: GlideMode,
}

impl Default for GlideParams {
    fn default() -> Self {
        Self {
            time: 0.0,
            mode: GlideMode::Off,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    pub name: String,
    pub oscillators: Vec<OscParams>,
    pub noise: NoiseParams,
    pub filter: FilterParams,
    pub filter_adsr: AdsrParams,
    pub amp_adsr: AdsrParams,
    pub lfo: LfoParams,
    pub glide: GlideParams,
    pub master_volume: f32,
}

impl Default for Patch {
    fn default() -> Self {
        Self {
            name: "Init".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    level: 0.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams::default(),
            filter: FilterParams::default(),
            filter_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.3,
                sustain: 0.2,
                release: 0.3,
            },
            amp_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.7,
                release: 0.3,
            },
            lfo: LfoParams::default(),
            glide: GlideParams::default(),
            master_volume: 0.7,
        }
    }
}
