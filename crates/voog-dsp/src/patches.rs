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
        Patch { name: "Init".to_string(), ..Patch::default() },
        // ── Classic ──
        // Fat detuned saw bass with filter envelope punch.
        Patch {
            name: "Bass Voog".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: -1, level: 1.0, ..Default::default() },
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
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 400.0,
                resonance: 0.4,
                env_amount: 24.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams { attack: 0.005, decay: 0.4, sustain: 0.1, release: 0.2 },
            amp_adsr: AdsrParams { attack: 0.005, decay: 0.3, sustain: 0.6, release: 0.15 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.8,
        },
        // Aggressive dual-saw lead with legato glide.
        Patch {
            name: "Lead Saw".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: 0, level: 1.0, ..Default::default() },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: 0,
                    detune: 12.0,
                    level: 0.8,
                    ..Default::default()
                },
                OscParams { waveform: Waveform::Saw, octave: 1, level: 0.3, ..Default::default() },
            ],
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 3000.0,
                resonance: 0.3,
                env_amount: 12.0,
                key_tracking: 0.7,
            },
            filter_adsr: AdsrParams { attack: 0.01, decay: 0.3, sustain: 0.3, release: 0.3 },
            amp_adsr: AdsrParams { attack: 0.01, decay: 0.1, sustain: 0.8, release: 0.3 },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.0,
                depth: 0.0,
                destination: LfoDest::Pitch,
                ..Default::default()
            },
            glide: GlideParams { time: 0.05, mode: GlideMode::Legato },
            master_volume: 0.7,
        },
        // Warm evolving pad with slow filter LFO.
        Patch {
            name: "Pad Strings".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: 0, level: 0.7, ..Default::default() },
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
            noise: NoiseParams { noise_type: NoiseType::Pink, level: 0.05 },
            filter: FilterParams {
                cutoff: 2000.0,
                resonance: 0.1,
                env_amount: 6.0,
                key_tracking: 0.3,
            },
            filter_adsr: AdsrParams { attack: 0.8, decay: 0.5, sustain: 0.6, release: 1.5 },
            amp_adsr: AdsrParams { attack: 0.6, decay: 0.3, sustain: 0.8, release: 1.5 },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.3,
                depth: 0.15,
                destination: LfoDest::Filter,
                ..Default::default()
            },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
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
                OscParams { waveform: Waveform::Saw, octave: -1, level: 0.0, ..Default::default() },
            ],
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 180.0,
                resonance: 0.15,
                env_amount: 12.0,
                key_tracking: 0.8,
            },
            filter_adsr: AdsrParams { attack: 0.003, decay: 0.25, sustain: 0.0, release: 0.12 },
            amp_adsr: AdsrParams { attack: 0.003, decay: 0.15, sustain: 0.9, release: 0.1 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.85,
        },
        // Resonant acid bass — high-Q filter with fast envelope sweep.
        Patch {
            name: "Acid Squelch".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: -1, level: 1.0, ..Default::default() },
                OscParams {
                    waveform: Waveform::Square,
                    octave: -1,
                    semitone: 0,
                    level: 0.5,
                    ..Default::default()
                },
                OscParams { waveform: Waveform::Saw, octave: 0, level: 0.0, ..Default::default() },
            ],
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 200.0,
                resonance: 0.75,
                env_amount: 36.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams { attack: 0.002, decay: 0.18, sustain: 0.0, release: 0.1 },
            amp_adsr: AdsrParams { attack: 0.002, decay: 0.5, sustain: 0.4, release: 0.12 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.04, mode: GlideMode::Always },
            master_volume: 0.75,
        },
        // Snappy percussive pluck — short decay, punchy attack.
        Patch {
            name: "Funky Pluck".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: 0, level: 1.0, ..Default::default() },
                OscParams {
                    waveform: Waveform::Square,
                    octave: 0,
                    detune: 5.0,
                    level: 0.6,
                    ..Default::default()
                },
                OscParams { waveform: Waveform::Saw, octave: -1, level: 0.3, ..Default::default() },
            ],
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 350.0,
                resonance: 0.35,
                env_amount: 30.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams { attack: 0.001, decay: 0.12, sustain: 0.0, release: 0.08 },
            amp_adsr: AdsrParams { attack: 0.001, decay: 0.25, sustain: 0.0, release: 0.08 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.75,
        },
        // Bright aggressive lead — full resonance, always-on glide.
        Patch {
            name: "Screaming Lead".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: 0, level: 1.0, ..Default::default() },
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
            noise: NoiseParams { level: 0.02, ..Default::default() },
            filter: FilterParams {
                cutoff: 1200.0,
                resonance: 0.55,
                env_amount: 18.0,
                key_tracking: 0.9,
            },
            filter_adsr: AdsrParams { attack: 0.01, decay: 0.4, sustain: 0.5, release: 0.35 },
            amp_adsr: AdsrParams { attack: 0.008, decay: 0.15, sustain: 0.85, release: 0.25 },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.5,
                depth: 0.08,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams { time: 0.06, mode: GlideMode::Always },
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
                OscParams { waveform: Waveform::Saw, octave: -1, level: 0.25, ..Default::default() },
            ],
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 800.0,
                resonance: 0.15,
                env_amount: 16.0,
                key_tracking: 0.7,
            },
            filter_adsr: AdsrParams { attack: 0.06, decay: 0.35, sustain: 0.45, release: 0.25 },
            amp_adsr: AdsrParams { attack: 0.04, decay: 0.2, sustain: 0.75, release: 0.2 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.7,
        },
        // Low detuned drone with slow filter LFO sweep.
        Patch {
            name: "Dark Drone".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: -1, level: 0.8, ..Default::default() },
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
            noise: NoiseParams { noise_type: NoiseType::Pink, level: 0.08 },
            filter: FilterParams {
                cutoff: 500.0,
                resonance: 0.45,
                env_amount: 4.0,
                key_tracking: 0.2,
            },
            filter_adsr: AdsrParams { attack: 1.0, decay: 0.5, sustain: 0.7, release: 2.0 },
            amp_adsr: AdsrParams { attack: 0.8, decay: 0.4, sustain: 0.85, release: 2.0 },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.15,
                depth: 0.25,
                destination: LfoDest::Filter,
                key_sync: false,
            },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
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
                OscParams { waveform: Waveform::Sine, octave: 1, level: 0.4, ..Default::default() },
                OscParams { waveform: Waveform::Saw, octave: 0, level: 0.0, ..Default::default() },
            ],
            noise: NoiseParams { noise_type: NoiseType::White, level: 0.3 },
            filter: FilterParams {
                cutoff: 600.0,
                resonance: 0.2,
                env_amount: 40.0,
                key_tracking: 0.4,
            },
            filter_adsr: AdsrParams { attack: 0.001, decay: 0.06, sustain: 0.0, release: 0.04 },
            amp_adsr: AdsrParams { attack: 0.001, decay: 0.15, sustain: 0.0, release: 0.06 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
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
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 2500.0,
                resonance: 0.1,
                env_amount: 10.0,
                key_tracking: 0.8,
            },
            filter_adsr: AdsrParams { attack: 0.001, decay: 0.6, sustain: 0.15, release: 0.4 },
            amp_adsr: AdsrParams { attack: 0.001, decay: 0.8, sustain: 0.2, release: 0.5 },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 4.0,
                depth: 0.03,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.7,
        },
        // Wobble bass — LFO-driven filter modulation, heavy low end.
        Patch {
            name: "Wobble Bass".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: -1, level: 1.0, ..Default::default() },
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
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 300.0,
                resonance: 0.5,
                env_amount: 6.0,
                key_tracking: 0.4,
            },
            filter_adsr: AdsrParams { attack: 0.005, decay: 0.2, sustain: 0.3, release: 0.15 },
            amp_adsr: AdsrParams { attack: 0.005, decay: 0.1, sustain: 0.9, release: 0.12 },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 3.0,
                depth: 0.6,
                destination: LfoDest::Filter,
                key_sync: true,
            },
            glide: GlideParams { time: 0.03, mode: GlideMode::Legato },
            master_volume: 0.75,
        },
        // Bright supersaw lead with pitch vibrato.
        Patch {
            name: "Trance Lead".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: 0, level: 1.0, ..Default::default() },
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
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 5000.0,
                resonance: 0.2,
                env_amount: 8.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams { attack: 0.01, decay: 0.4, sustain: 0.4, release: 0.35 },
            amp_adsr: AdsrParams { attack: 0.01, decay: 0.1, sustain: 0.85, release: 0.3 },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 5.8,
                depth: 0.06,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
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
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 4000.0,
                resonance: 0.15,
                env_amount: 10.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams { attack: 0.01, decay: 0.5, sustain: 0.3, release: 0.4 },
            amp_adsr: AdsrParams { attack: 0.01, decay: 0.15, sustain: 0.8, release: 0.35 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.6,
        },
        // Resonant sweep — self-oscillating filter with slow envelope.
        Patch {
            name: "Reso Sweep".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: 0, level: 0.7, ..Default::default() },
                OscParams {
                    waveform: Waveform::Saw,
                    octave: -1,
                    detune: 6.0,
                    level: 0.5,
                    ..Default::default()
                },
                OscParams { waveform: Waveform::Saw, octave: 0, level: 0.0, ..Default::default() },
            ],
            noise: NoiseParams { noise_type: NoiseType::Pink, level: 0.04 },
            filter: FilterParams {
                cutoff: 120.0,
                resonance: 0.85,
                env_amount: 44.0,
                key_tracking: 0.6,
            },
            filter_adsr: AdsrParams { attack: 0.005, decay: 1.5, sustain: 0.0, release: 0.8 },
            amp_adsr: AdsrParams { attack: 0.005, decay: 1.8, sustain: 0.0, release: 0.8 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.65,
        },
        // Power fifth stab — osc2 tuned to a fifth, short punchy hit.
        Patch {
            name: "Fifth Stab".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: 0, level: 1.0, ..Default::default() },
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
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 500.0,
                resonance: 0.25,
                env_amount: 28.0,
                key_tracking: 0.5,
            },
            filter_adsr: AdsrParams { attack: 0.001, decay: 0.15, sustain: 0.05, release: 0.1 },
            amp_adsr: AdsrParams { attack: 0.001, decay: 0.3, sustain: 0.0, release: 0.1 },
            lfo: LfoParams { depth: 0.0, ..Default::default() },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.7,
        },
        // Crystalline bell tone — sine harmonics with long release.
        Patch {
            name: "Glass Bell".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Sine, octave: 0, level: 1.0, ..Default::default() },
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
            noise: NoiseParams { level: 0.0, ..Default::default() },
            filter: FilterParams {
                cutoff: 6000.0,
                resonance: 0.3,
                env_amount: 12.0,
                key_tracking: 0.9,
            },
            filter_adsr: AdsrParams { attack: 0.001, decay: 1.2, sustain: 0.1, release: 1.5 },
            amp_adsr: AdsrParams { attack: 0.001, decay: 1.5, sustain: 0.05, release: 2.0 },
            lfo: LfoParams {
                waveform: Waveform::Sine,
                rate: 3.5,
                depth: 0.02,
                destination: LfoDest::Pitch,
                key_sync: true,
            },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.7,
        },
        // Noise texture — filtered noise with resonant sweep, sci-fi flavor.
        Patch {
            name: "Noise Sweep".to_string(),
            oscillators: vec![
                OscParams { waveform: Waveform::Saw, octave: 0, level: 0.2, ..Default::default() },
                OscParams { waveform: Waveform::Saw, octave: 0, level: 0.0, ..Default::default() },
                OscParams { waveform: Waveform::Saw, octave: 0, level: 0.0, ..Default::default() },
            ],
            noise: NoiseParams { noise_type: NoiseType::White, level: 0.8 },
            filter: FilterParams {
                cutoff: 150.0,
                resonance: 0.7,
                env_amount: 40.0,
                key_tracking: 0.0,
            },
            filter_adsr: AdsrParams { attack: 0.3, decay: 1.0, sustain: 0.2, release: 0.8 },
            amp_adsr: AdsrParams { attack: 0.2, decay: 0.8, sustain: 0.4, release: 1.0 },
            lfo: LfoParams {
                waveform: Waveform::Triangle,
                rate: 0.2,
                depth: 0.3,
                destination: LfoDest::Filter,
                key_sync: false,
            },
            glide: GlideParams { time: 0.0, mode: GlideMode::Off },
            master_volume: 0.6,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_expected_count() {
        // 19 presets: Init + 3 classic + 15 Subsequent-37 style.
        assert_eq!(factory_presets().len(), 19);
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
            assert_eq!(patch.oscillators.len(), 3, "preset '{}' needs 3 oscillators", patch.name);
        }
    }

    #[test]
    fn all_fields_finite_and_levels_in_range() {
        let in_unit = |v: f32| (0.0..=1.0).contains(&v);
        for patch in factory_presets() {
            for osc in &patch.oscillators {
                assert!(osc.detune.is_finite());
                assert!(osc.level.is_finite() && in_unit(osc.level), "bad osc level in '{}'", patch.name);
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
