//! Band-limited wavetables built once via additive synthesis.
//! Ported from `_build_tables()` in `synth/dsp/oscillator.py`.

use crate::config::WAVETABLE_SIZE;
use crate::params::Waveform;
use std::f32::consts::PI;
use std::sync::OnceLock;

pub struct Wavetables {
    pub sine: Vec<f32>,
    pub saw: Vec<f32>,
    pub square: Vec<f32>,
    pub triangle: Vec<f32>,
}

fn build() -> Wavetables {
    let n = WAVETABLE_SIZE;
    let mut sine = vec![0.0f32; n];
    let mut saw = vec![0.0f32; n];
    let mut square = vec![0.0f32; n];
    let mut triangle = vec![0.0f32; n];

    for (i, ((s, sw), (sq, tr))) in sine
        .iter_mut()
        .zip(saw.iter_mut())
        .zip(square.iter_mut().zip(triangle.iter_mut()))
        .enumerate()
    {
        let phase = i as f32 / n as f32; // linspace(0,1, endpoint=False)
        *s = (2.0 * PI * phase).sin();

        // Saw: additive, harmonics 1..63
        let mut acc = 0.0f32;
        for k in 1..64 {
            let sign = if (k + 1) % 2 == 0 { 1.0 } else { -1.0 };
            acc += sign * (2.0 * PI * k as f32 * phase).sin() / k as f32;
        }
        *sw = acc * (2.0 / PI);

        // Square: odd harmonics
        let mut acc = 0.0f32;
        let mut k = 1;
        while k < 64 {
            acc += (2.0 * PI * k as f32 * phase).sin() / k as f32;
            k += 2;
        }
        *sq = acc * (4.0 / PI);

        // Triangle: odd harmonics, alternating sign, 1/k^2
        let mut acc = 0.0f32;
        let mut k = 1;
        while k < 64 {
            let sign = if (((k - 1) / 2) % 2) == 0 { 1.0 } else { -1.0 };
            acc += sign * (2.0 * PI * k as f32 * phase).sin() / (k * k) as f32;
            k += 2;
        }
        *tr = acc * (8.0 / (PI * PI));
    }

    Wavetables {
        sine,
        saw,
        square,
        triangle,
    }
}

static TABLES: OnceLock<Wavetables> = OnceLock::new();

/// Borrow the shared band-limited table for a waveform (built lazily, once).
pub fn wavetable(w: Waveform) -> &'static [f32] {
    let t = TABLES.get_or_init(build);
    match w {
        Waveform::Sine => &t.sine,
        Waveform::Saw => &t.saw,
        Waveform::Square => &t.square,
        Waveform::Triangle => &t.triangle,
    }
}
