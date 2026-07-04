//! Huovilainen Moog ladder filter (24 dB/oct). PORT OF `synth/dsp/filter.py`
//! (the numba `_moog_ladder_process` reference implementation).
//!
//! Implement `process`: sample-by-sample, in place. `resonance` is 0..1 and
//! maps to feedback `r = resonance * 4.0`. Clamp cutoff to `SAMPLE_RATE*0.49`.

pub struct MoogFilter {
    pub cutoff: f32,       // Hz
    pub resonance: f32,    // 0..1
    pub env_amount: f32,   // semitones (used by the voice layer)
    pub key_tracking: f32, // 0..1 (used by the voice layer)
    state: [f32; 4],
}

impl Default for MoogFilter {
    fn default() -> Self {
        Self { cutoff: 8000.0, resonance: 0.0, env_amount: 0.0, key_tracking: 0.0, state: [0.0; 4] }
    }
}

impl MoogFilter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process `io` in place. `cutoff_mod`, if present (len == io.len()), is a
    /// per-sample Hz offset ADDED to `self.cutoff` before clamping to 20..sr*0.49.
    pub fn process(&mut self, io: &mut [f32], cutoff_mod: Option<&[f32]>) {
        let _ = (io, cutoff_mod, &self.state);
        todo!("port _moog_ladder_process from synth/dsp/filter.py")
    }

    pub fn reset(&mut self) {
        self.state = [0.0; 4];
    }
}
