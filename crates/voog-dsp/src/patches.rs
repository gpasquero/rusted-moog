//! Built-in factory presets. PORT OF `synth/patch/default_patches.py`.
//!
//! Faithful port of the 19 named presets defined in the Python source, in the
//! same order. Each `Patch.name` is the preset name.

use crate::params::{
    AdsrParams, FilterParams, GlideMode, GlideParams, LfoDest, LfoParams, NoiseParams, NoiseType,
    OscParams, Patch, Waveform,
};

/// All factory presets, in display order. Each `Patch.name` is the preset name.
pub fn factory_presets() -> Vec<Patch> {
    vec![
        // ── Init ──
        Patch {
            name: "Init".to_string(),
            ..Patch::default()
        },
        // ── Classic ──
        // Fat detuned saw bass with filter envelope punch.
        Patch {
            name: "Bass Voog".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    detune: 8.0,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -2,
                    level: 0.5,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 400.0,
                resonance: 0.4,
                env_amount: 24.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.005,
                decay: 0.4,
                sustain: 0.1,
                release: 0.2,
            },
            amp_adsr: AdsrParams {
                attack: 0.005,
                decay: 0.3,
                sustain: 0.6,
                release: 0.15,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.8,
        },
        // Aggressive dual-saw lead with legato glide.
        Patch {
            name: "Lead Saw".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 12.0,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 1,
                    level: 0.3,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 3000.0,
                resonance: 0.3,
                env_amount: 12.0,
                key_tracking: 0.7,
            },
            filter_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.3,
                sustain: 0.3,
                release: 0.3,
            },
            amp_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.8,
                release: 0.3,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.0,
                depth: 0.0,
                destination: LfoDest::Pitch,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.05,
                mode: GlideMode::Legato,
            },
            master_volume: 0.7,
        },
        // Warm evolving pad with slow filter LFO.
        Patch {
            name: "Pad Strings".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 7.0,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 1,
                    detune: -5.0,
                    level: 0.4,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::Pink,
                level: 0.05,
            },
            filter: FilterParams {
                cutoff: 2000.0,
                resonance: 0.1,
                env_amount: 6.0,
                key_tracking: 0.3,
            },
            filter_adsr: AdsrParams {
                attack: 0.8,
                decay: 0.5,
                sustain: 0.6,
                release: 1.5,
            },
            amp_adsr: AdsrParams {
                attack: 0.6,
                decay: 0.3,
                sustain: 0.8,
                release: 1.5,
            },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.3,
                depth: 0.15,
                destination: LfoDest::Filter,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.6,
        },
        // ── Subsequent 37 ──
        // Deep sub bass — pure low-end weight with square sub layer.
        Patch {
            name: "Sub Thunder".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Square,
                    octave: -2,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: -1,
                    level: 0.4,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 180.0,
                resonance: 0.15,
                env_amount: 12.0,
                key_tracking: 0.8,
            },
            filter_adsr: AdsrParams {
                attack: 0.003,
                decay: 0.25,
                sustain: 0.0,
                release: 0.12,
            },
            amp_adsr: AdsrParams {
                attack: 0.003,
                decay: 0.15,
                sustain: 0.9,
                release: 0.1,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.85,
        },
        // Resonant acid bass — high-Q filter with fast envelope sweep.
        Patch {
            name: "Acid Squelch".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -1,
                    semitone: 0,
                    level: 0.5,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 200.0,
                resonance: 0.75,
                env_amount: 36.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.002,
                decay: 0.18,
                sustain: 0.0,
                release: 0.1,
            },
            amp_adsr: AdsrParams {
                attack: 0.002,
                decay: 0.5,
                sustain: 0.4,
                release: 0.12,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.04,
                mode: GlideMode::Always,
            },
            master_volume: 0.75,
        },
        // Snappy percussive pluck — short decay, punchy attack.
        Patch {
            name: "Funky Pluck".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    detune: 5.0,
                    level: 0.6,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 0.3,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 350.0,
                resonance: 0.35,
                env_amount: 30.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.12,
                sustain: 0.0,
                release: 0.08,
            },
            amp_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.25,
                sustain: 0.0,
                release: 0.08,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.75,
        },
        // Bright aggressive lead — full resonance, always-on glide.
        Patch {
            name: "Screaming Lead".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 15.0,
                    level: 0.9,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 1,
                    level: 0.4,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.02,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 1200.0,
                resonance: 0.55,
                env_amount: 18.0,
                key_tracking: 0.9,
            },
            filter_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.4,
                sustain: 0.5,
                release: 0.35,
            },
            amp_adsr: AdsrParams {
                attack: 0.008,
                decay: 0.15,
                sustain: 0.85,
                release: 0.25,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.5,
                depth: 0.08,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.06,
                mode: GlideMode::Always,
            },
            master_volume: 0.7,
        },
        // Mellow brass tone — square/saw mix with medium attack.
        Patch {
            name: "Warm Brass".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    level: 0.9,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 3.0,
                    level: 0.6,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 0.25,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 800.0,
                resonance: 0.15,
                env_amount: 16.0,
                key_tracking: 0.7,
            },
            filter_adsr: AdsrParams {
                attack: 0.06,
                decay: 0.35,
                sustain: 0.45,
                release: 0.25,
            },
            amp_adsr: AdsrParams {
                attack: 0.04,
                decay: 0.2,
                sustain: 0.75,
                release: 0.2,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.7,
        },
        // Low detuned drone with slow filter LFO sweep.
        Patch {
            name: "Dark Drone".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    detune: -10.0,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -2,
                    detune: 5.0,
                    level: 0.5,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::Pink,
                level: 0.08,
            },
            filter: FilterParams {
                cutoff: 500.0,
                resonance: 0.45,
                env_amount: 4.0,
                key_tracking: 0.2,
            },
            filter_adsr: AdsrParams {
                attack: 1.0,
                decay: 0.5,
                sustain: 0.7,
                release: 2.0,
            },
            amp_adsr: AdsrParams {
                attack: 0.8,
                decay: 0.4,
                sustain: 0.85,
                release: 2.0,
            },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.15,
                depth: 0.25,
                destination: LfoDest::Filter,
                key_sync: false,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.6,
        },
        // Percussive transient hit — noise burst with pitched body.
        Patch {
            name: "Perc Hit".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 0,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 1,
                    level: 0.4,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::White,
                level: 0.3,
            },
            filter: FilterParams {
                cutoff: 600.0,
                resonance: 0.2,
                env_amount: 40.0,
                key_tracking: 0.4,
            },
            filter_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.06,
                sustain: 0.0,
                release: 0.04,
            },
            amp_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.15,
                sustain: 0.0,
                release: 0.06,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.75,
        },
        // Vintage electric piano tone — triangle/square mix, bell-like decay.
        Patch {
            name: "Vintage Keys".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 1,
                    level: 0.2,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 0,
                    detune: 1.5,
                    level: 0.3,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 2500.0,
                resonance: 0.1,
                env_amount: 10.0,
                key_tracking: 0.8,
            },
            filter_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.6,
                sustain: 0.15,
                release: 0.4,
            },
            amp_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.8,
                sustain: 0.2,
                release: 0.5,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 4.0,
                depth: 0.03,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.7,
        },
        // Wobble bass — LFO-driven filter modulation, heavy low end.
        Patch {
            name: "Wobble Bass".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -1,
                    detune: 4.0,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -2,
                    level: 0.6,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 300.0,
                resonance: 0.5,
                env_amount: 6.0,
                key_tracking: 0.4,
            },
            filter_adsr: AdsrParams {
                attack: 0.005,
                decay: 0.2,
                sustain: 0.3,
                release: 0.15,
            },
            amp_adsr: AdsrParams {
                attack: 0.005,
                decay: 0.1,
                sustain: 0.9,
                release: 0.12,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 3.0,
                depth: 0.6,
                destination: LfoDest::Filter,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.03,
                mode: GlideMode::Legato,
            },
            master_volume: 0.75,
        },
        // Bright supersaw lead with pitch vibrato.
        Patch {
            name: "Trance Lead".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 20.0,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: -18.0,
                    level: 0.8,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 5000.0,
                resonance: 0.2,
                env_amount: 8.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.4,
                sustain: 0.4,
                release: 0.35,
            },
            amp_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.1,
                sustain: 0.85,
                release: 0.3,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.8,
                depth: 0.06,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.65,
        },
        // Massive detuned unison — three saws wide-spread, full spectrum.
        Patch {
            name: "Fat Unison".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: -25.0,
                    level: 0.9,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 0.0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 25.0,
                    level: 0.9,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 4000.0,
                resonance: 0.15,
                env_amount: 10.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.5,
                sustain: 0.3,
                release: 0.4,
            },
            amp_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.15,
                sustain: 0.8,
                release: 0.35,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.6,
        },
        // Resonant sweep — self-oscillating filter with slow envelope.
        Patch {
            name: "Reso Sweep".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    detune: 6.0,
                    level: 0.5,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::Pink,
                level: 0.04,
            },
            filter: FilterParams {
                cutoff: 120.0,
                resonance: 0.85,
                env_amount: 44.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams {
                attack: 0.005,
                decay: 1.5,
                sustain: 0.0,
                release: 0.8,
            },
            amp_adsr: AdsrParams {
                attack: 0.005,
                decay: 1.8,
                sustain: 0.0,
                release: 0.8,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.65,
        },
        // Power fifth stab — osc2 tuned to a fifth, short punchy hit.
        Patch {
            name: "Fifth Stab".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    semitone: 7,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -1,
                    level: 0.4,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 500.0,
                resonance: 0.25,
                env_amount: 28.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.15,
                sustain: 0.05,
                release: 0.1,
            },
            amp_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.3,
                sustain: 0.0,
                release: 0.1,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.7,
        },
        // Crystalline bell tone — sine harmonics with long release.
        Patch {
            name: "Glass Bell".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 1,
                    level: 0.35,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 2,
                    detune: 2.0,
                    level: 0.15,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 6000.0,
                resonance: 0.3,
                env_amount: 12.0,
                key_tracking: 0.9,
            },
            filter_adsr: AdsrParams {
                attack: 0.001,
                decay: 1.2,
                sustain: 0.1,
                release: 1.5,
            },
            amp_adsr: AdsrParams {
                attack: 0.001,
                decay: 1.5,
                sustain: 0.05,
                release: 2.0,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 3.5,
                depth: 0.02,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.7,
        },
        // Noise texture — filtered noise with resonant sweep, sci-fi flavor.
        Patch {
            name: "Noise Sweep".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.2,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::White,
                level: 0.8,
            },
            filter: FilterParams {
                cutoff: 150.0,
                resonance: 0.7,
                env_amount: 40.0,
                key_tracking: 0.0,
            },
            filter_adsr: AdsrParams {
                attack: 0.3,
                decay: 1.0,
                sustain: 0.2,
                release: 0.8,
            },
            amp_adsr: AdsrParams {
                attack: 0.2,
                decay: 0.8,
                sustain: 0.4,
                release: 1.0,
            },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.2,
                depth: 0.3,
                destination: LfoDest::Filter,
                key_sync: false,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.6,
        },
        // ── Documented / artist patches (researched) ──
        // Parliament "Flash Light" synth bass by Bernie Worrell: OSC1 32' saw,
        // OSC2 16' pulse pitched slightly up, a touch of noise, snappy filter
        // envelope (attack 0, decay ~400 ms, sustain 0), fast amp, short glide.
        // source: https://www.syntorial.com/preset-recipe/parliament-funkadelic-flashlight-bass/
        Patch {
            name: "Flashlight Funk".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -2,
                    level: 0.6,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -1,
                    detune: 5.0,
                    level: 0.4,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: -1,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::White,
                level: 0.05,
            },
            filter: FilterParams {
                cutoff: 405.0,
                resonance: 0.35,
                env_amount: 10.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.4,
                sustain: 0.0,
                release: 0.05,
            },
            amp_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.2,
                sustain: 1.0,
                release: 0.015,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.03,
                mode: GlideMode::Always,
            },
            master_volume: 0.85,
        },
        // Moog Taurus pedal-bass emulation: single 32' oscillator, filter nearly
        // closed so the output approaches a sine, ~50% keyboard tracking, slight
        // VCA attack, full sustain, slightly slower release.
        // source: https://forum.moogmusic.com/t/patch-to-emulate-taurus/11776
        Patch {
            name: "Taurus Pedal".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Square,
                    octave: -2,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -2,
                    level: 0.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -2,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 120.0,
                resonance: 0.05,
                env_amount: 4.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.02,
                decay: 0.3,
                sustain: 0.8,
                release: 0.3,
            },
            amp_adsr: AdsrParams {
                attack: 0.02,
                decay: 0.2,
                sustain: 1.0,
                release: 0.3,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.85,
        },
        // ELP "Lucky Man"-style ribbon lead: a single bright saw/triangle voice
        // with heavy always-on portamento for those swooping glissando runs and
        // a fairly open filter.
        // source: https://groups.google.com/d/topic/rec.music.makers.synth/BkeSxuvoEUs
        Patch {
            name: "Lucky Man Solo".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 0,
                    detune: 4.0,
                    level: 0.5,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 1,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 2600.0,
                resonance: 0.2,
                env_amount: 8.0,
                key_tracking: 0.7,
            },
            filter_adsr: AdsrParams {
                attack: 0.02,
                decay: 0.4,
                sustain: 0.6,
                release: 0.3,
            },
            amp_adsr: AdsrParams {
                attack: 0.02,
                decay: 0.2,
                sustain: 0.9,
                release: 0.25,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.0,
                depth: 0.04,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.12,
                mode: GlideMode::Always,
            },
            master_volume: 0.7,
        },
        // Rick Wakeman "Catherine of Aragon"-style lead: two same-octave (8')
        // saws, never mixed waveforms, only slightly detuned, cutoff toward the
        // low end for that vintage vocal-ish lead.
        // source: https://gearspace.com/board/electronic-music-instruments-and-electronic-music-production/651302-rick-wakemans-catherine-aragon-patch-moog-minimoog.html
        Patch {
            name: "Aragon Lead".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 6.0,
                    level: 0.9,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: -6.0,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 900.0,
                resonance: 0.3,
                env_amount: 14.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams {
                attack: 0.02,
                decay: 0.4,
                sustain: 0.5,
                release: 0.3,
            },
            amp_adsr: AdsrParams {
                attack: 0.015,
                decay: 0.2,
                sustain: 0.85,
                release: 0.25,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.2,
                depth: 0.05,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.04,
                mode: GlideMode::Legato,
            },
            master_volume: 0.7,
        },
        // Flute/whistle: the Minimoog manual notes the triangle wave gives soft,
        // pure flute-like tones. Nearly pure triangle, a breath of pink noise,
        // gentle attack and a little vibrato.
        // source: https://cdn.inmusicbrands.com/Moog/Model%20D/Minimoog_Model_D_Manual.pdf
        Patch {
            name: "Silver Flute".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 1,
                    level: 0.15,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 0,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::Pink,
                level: 0.06,
            },
            filter: FilterParams {
                cutoff: 2200.0,
                resonance: 0.05,
                env_amount: 6.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams {
                attack: 0.06,
                decay: 0.3,
                sustain: 0.7,
                release: 0.25,
            },
            amp_adsr: AdsrParams {
                attack: 0.05,
                decay: 0.2,
                sustain: 0.85,
                release: 0.2,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.0,
                depth: 0.05,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.7,
        },
        // High whistle lead: nearly pure sine an octave up with expressive
        // vibrato, wide-open filter — the classic reggae/dub "whistle" lead.
        // source: https://cdn.inmusicbrands.com/Moog/Model%20D/Minimoog_Model_D_Manual.pdf
        Patch {
            name: "Whistle Top".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 1,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 1,
                    detune: 3.0,
                    level: 0.2,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 2,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 6000.0,
                resonance: 0.1,
                env_amount: 4.0,
                key_tracking: 0.8,
            },
            filter_adsr: AdsrParams {
                attack: 0.03,
                decay: 0.2,
                sustain: 0.8,
                release: 0.2,
            },
            amp_adsr: AdsrParams {
                attack: 0.03,
                decay: 0.15,
                sustain: 0.9,
                release: 0.2,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 6.0,
                depth: 0.09,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.05,
                mode: GlideMode::Legato,
            },
            master_volume: 0.65,
        },
        // Brass ensemble: slightly detuned saws and pulses across all three
        // oscillators, moderate cutoff and low resonance, with the medium
        // attack/long-ish sweep that gives soft brass its character.
        // source: https://avarethtaika.com/2023/03/20/minimoog-sound-design/
        Patch {
            name: "Brass Ensemble".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.9,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    detune: 6.0,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: -8.0,
                    level: 0.6,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 1100.0,
                resonance: 0.12,
                env_amount: 20.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.12,
                decay: 0.4,
                sustain: 0.6,
                release: 0.35,
            },
            amp_adsr: AdsrParams {
                attack: 0.08,
                decay: 0.25,
                sustain: 0.8,
                release: 0.3,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.65,
        },
        // Funky clav: bright square with high key-tracking and resonance, very
        // fast plucky envelope for that percussive Clavinet-style stab.
        // source: https://www.presetpatch.com/synth/moog-minimoog
        Patch {
            name: "Funky Clav".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    detune: 4.0,
                    level: 0.5,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 0.3,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 800.0,
                resonance: 0.45,
                env_amount: 26.0,
                key_tracking: 0.9,
            },
            filter_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.1,
                sustain: 0.0,
                release: 0.08,
            },
            amp_adsr: AdsrParams {
                attack: 0.001,
                decay: 0.18,
                sustain: 0.0,
                release: 0.08,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.75,
        },
        // Rock organ: stacked square waves at octaves, fast attack, high sustain
        // and no filter movement — a drawbar-style tone-wheel emulation.
        // source: http://www.synthzone.com/midi/moog/minimoog/MINIMOOG%20PATCH%20BOOK.pdf
        Patch {
            name: "Rock Organ".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 1,
                    level: 0.6,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -1,
                    level: 0.5,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 4000.0,
                resonance: 0.05,
                env_amount: 0.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.005,
                decay: 0.1,
                sustain: 1.0,
                release: 0.1,
            },
            amp_adsr: AdsrParams {
                attack: 0.005,
                decay: 0.1,
                sustain: 1.0,
                release: 0.1,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 6.5,
                depth: 0.03,
                destination: LfoDest::Pitch,
                key_sync: false,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.65,
        },
        // Hard-sync-flavored scream: OSC2 pushed sharp by a semitone plus detune
        // to emulate the clangy overtones of oscillator sync, high resonance and
        // heavy filter envelope.
        // source: https://reverb.com/news/video-the-synth-sounds-of-5-classic-minimoog-tracks
        Patch {
            name: "Sync Screamer".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    semitone: 7,
                    detune: 12.0,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 1,
                    semitone: 3,
                    level: 0.5,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 1500.0,
                resonance: 0.6,
                env_amount: 24.0,
                key_tracking: 0.8,
            },
            filter_adsr: AdsrParams {
                attack: 0.01,
                decay: 0.3,
                sustain: 0.4,
                release: 0.25,
            },
            amp_adsr: AdsrParams {
                attack: 0.008,
                decay: 0.15,
                sustain: 0.8,
                release: 0.2,
            },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.5,
                depth: 0.06,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams {
                time: 0.03,
                mode: GlideMode::Legato,
            },
            master_volume: 0.65,
        },
        // FM-style metallic bell: oscillators tuned to an inharmonic interval
        // (major third + detune) to fake FM sidebands, bright open filter and a
        // long bell-like decay.
        // source: https://avarethtaika.com/2023/03/20/minimoog-sound-design/
        Patch {
            name: "Metal FM".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    level: 0.9,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 1,
                    semitone: 4,
                    detune: 4.0,
                    level: 0.5,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: 2,
                    semitone: -5,
                    level: 0.3,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 5000.0,
                resonance: 0.25,
                env_amount: 14.0,
                key_tracking: 0.9,
            },
            filter_adsr: AdsrParams {
                attack: 0.001,
                decay: 1.0,
                sustain: 0.1,
                release: 1.2,
            },
            amp_adsr: AdsrParams {
                attack: 0.001,
                decay: 1.4,
                sustain: 0.05,
                release: 1.6,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.65,
        },
        // Arpeggio stab: pad-style oscillator stack but with short, plucky
        // envelopes — the classic trance/house arpeggiator stab.
        // source: https://avarethtaika.com/2023/03/20/minimoog-sound-design/
        Patch {
            name: "Arp Stab".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 0.9,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    detune: 8.0,
                    level: 0.6,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 1,
                    level: 0.4,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 900.0,
                resonance: 0.4,
                env_amount: 24.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams {
                attack: 0.002,
                decay: 0.18,
                sustain: 0.0,
                release: 0.12,
            },
            amp_adsr: AdsrParams {
                attack: 0.002,
                decay: 0.2,
                sustain: 0.0,
                release: 0.12,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.7,
        },
        // Classic house organ bass: stacked squares at octaves, minimum cutoff
        // with resonance and envelope amount for a punchy, hollow low end.
        // source: https://www.musicradar.com/how-to/minimoog-analogue-bass
        Patch {
            name: "House Bass".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Square,
                    octave: -1,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -2,
                    level: 0.7,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    detune: 5.0,
                    level: 0.4,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 250.0,
                resonance: 0.5,
                env_amount: 20.0,
                key_tracking: 0.4,
            },
            filter_adsr: AdsrParams {
                attack: 0.002,
                decay: 0.15,
                sustain: 0.2,
                release: 0.1,
            },
            amp_adsr: AdsrParams {
                attack: 0.002,
                decay: 0.12,
                sustain: 0.7,
                release: 0.1,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.8,
        },
        // Techno reso stab: aggressive detuned saws through a high-resonance
        // filter with a big, fast envelope sweep — a driving techno hit.
        // source: https://www.musicradar.com/how-to/minimoog-analogue-bass
        Patch {
            name: "Techno Stab".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    detune: 10.0,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    level: 0.5,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 300.0,
                resonance: 0.7,
                env_amount: 40.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.002,
                decay: 0.22,
                sustain: 0.0,
                release: 0.12,
            },
            amp_adsr: AdsrParams {
                attack: 0.002,
                decay: 0.3,
                sustain: 0.0,
                release: 0.12,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.75,
        },
        // Pure sine sub bass: a single sine an octave-and-two down for clean,
        // deep low-end weight with no harmonics — the modern sub-bass staple.
        // source: https://www.musicradar.com/how-to/minimoog-analogue-bass
        Patch {
            name: "Deep Sub Sine".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Sine,
                    octave: -2,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: -1,
                    level: 0.2,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Sine,
                    octave: -2,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 400.0,
                resonance: 0.0,
                env_amount: 6.0,
                key_tracking: 0.7,
            },
            filter_adsr: AdsrParams {
                attack: 0.005,
                decay: 0.2,
                sustain: 0.6,
                release: 0.15,
            },
            amp_adsr: AdsrParams {
                attack: 0.005,
                decay: 0.15,
                sustain: 1.0,
                release: 0.12,
            },
            lfo: LfoParams {
                depth: 0.0,
                ..Default::default()
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.85,
        },
        // Analog strings ensemble: three slightly detuned saws with slow attack
        // and long release, gentle filter LFO for the shimmering ensemble sweep.
        // source: https://avarethtaika.com/2023/03/20/minimoog-sound-design/
        Patch {
            name: "Analog Strings".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: -9.0,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 9.0,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 1,
                    detune: -4.0,
                    level: 0.45,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::Pink,
                level: 0.03,
            },
            filter: FilterParams {
                cutoff: 1800.0,
                resonance: 0.12,
                env_amount: 8.0,
                key_tracking: 0.4,
            },
            filter_adsr: AdsrParams {
                attack: 0.7,
                decay: 0.5,
                sustain: 0.7,
                release: 1.4,
            },
            amp_adsr: AdsrParams {
                attack: 0.5,
                decay: 0.3,
                sustain: 0.85,
                release: 1.3,
            },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.4,
                depth: 0.12,
                destination: LfoDest::Filter,
                key_sync: false,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.6,
        },
        // Evolving space drone: detuned low saws with a very slow filter LFO and
        // long envelopes for an ambient, cinematic bed.
        // source: https://avarethtaika.com/2023/03/20/minimoog-sound-design/
        Patch {
            name: "Space Drone".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 0,
                    detune: 7.0,
                    level: 0.6,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -2,
                    detune: -6.0,
                    level: 0.5,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                noise_type: NoiseType::Pink,
                level: 0.06,
            },
            filter: FilterParams {
                cutoff: 700.0,
                resonance: 0.35,
                env_amount: 6.0,
                key_tracking: 0.3,
            },
            filter_adsr: AdsrParams {
                attack: 1.5,
                decay: 0.6,
                sustain: 0.75,
                release: 2.5,
            },
            amp_adsr: AdsrParams {
                attack: 1.2,
                decay: 0.5,
                sustain: 0.85,
                release: 2.5,
            },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.12,
                depth: 0.3,
                destination: LfoDest::Filter,
                key_sync: false,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.55,
        },
        // Vocal "formant" patch: VCO1 32' saw at max, filter around 25% cutoff
        // and ~60% resonance to accentuate a formant peak, with a slow filter LFO
        // standing in for the osc3 filter modulation of the original recipe.
        // source: https://avarethtaika.com/2023/03/20/minimoog-sound-design/
        Patch {
            name: "Vocal Formant".to_string(),
            oscillators: vec![
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    level: 1.0,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 5.0,
                    level: 0.4,
                    ..Default::default()
                },
                OscParams {
                    waveform: Waveform::Triangle,
                    octave: 0,
                    level: 0.0,
                    ..Default::default()
                },
            ],
            noise: NoiseParams {
                level: 0.0,
                ..Default::default()
            },
            filter: FilterParams {
                cutoff: 110.0,
                resonance: 0.6,
                env_amount: 18.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams {
                attack: 0.05,
                decay: 0.4,
                sustain: 0.5,
                release: 0.3,
            },
            amp_adsr: AdsrParams {
                attack: 0.04,
                decay: 0.3,
                sustain: 0.8,
                release: 0.3,
            },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.8,
                depth: 0.25,
                destination: LfoDest::Filter,
                key_sync: false,
            },
            glide: GlideParams {
                time: 0.0,
                mode: GlideMode::Off,
            },
            master_volume: 0.65,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_expected_count() {
        // 37 presets: original 19 + 18 researched/documented artist patches.
        assert_eq!(factory_presets().len(), 37);
    }

    #[test]
    fn names_unique_and_non_empty() {
        let presets = factory_presets();
        let mut names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
        for name in &names {
            assert!(!name.is_empty(), "preset name must not be empty");
        }
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count, "preset names must be unique");
    }

    #[test]
    fn every_preset_has_three_oscillators() {
        for patch in factory_presets() {
            assert_eq!(
                patch.oscillators.len(),
                3,
                "preset '{}' needs 3 oscillators",
                patch.name
            );
        }
    }

    #[test]
    fn all_fields_finite_and_levels_in_range() {
        let in_unit = |v: f32| (0.0..=1.0).contains(&v);
        for patch in factory_presets() {
            for osc in &patch.oscillators {
                assert!(osc.detune.is_finite());
                assert!(
                    osc.level.is_finite() && in_unit(osc.level),
                    "bad osc level in '{}'",
                    patch.name
                );
                assert!(osc.pulse_width.is_finite() && in_unit(osc.pulse_width));
            }
            assert!(patch.noise.level.is_finite() && in_unit(patch.noise.level));

            assert!(patch.filter.cutoff.is_finite());
            assert!(patch.filter.resonance.is_finite() && in_unit(patch.filter.resonance));
            assert!(patch.filter.env_amount.is_finite());
            assert!(patch.filter.key_tracking.is_finite() && in_unit(patch.filter.key_tracking));

            for adsr in [&patch.filter_adsr, &patch.amp_adsr] {
                assert!(adsr.attack.is_finite());
                assert!(adsr.decay.is_finite());
                assert!(adsr.sustain.is_finite() && in_unit(adsr.sustain));
                assert!(adsr.release.is_finite());
            }

            assert!(patch.lfo.rate.is_finite());
            assert!(patch.lfo.depth.is_finite());
            assert!(patch.glide.time.is_finite());

            assert!(
                patch.master_volume.is_finite() && in_unit(patch.master_volume),
                "bad master_volume in '{}'",
                patch.name
            );
        }
    }
}
