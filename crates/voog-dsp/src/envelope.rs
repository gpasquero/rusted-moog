//! ADSR envelope. PORT OF `synth/dsp/envelope.py`.
//!
//! Compute at control rate (blocks of `CONTROL_RATE_DIVIDER`) then linearly
//! interpolate to audio rate, exactly like the Python `render`. `process`
//! OVERWRITES `out` with the per-sample envelope (0..1).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct Adsr {
    pub attack: f32,  // seconds
    pub decay: f32,   // seconds
    pub sustain: f32, // 0..1
    pub release: f32, // seconds
    state: EnvState,
    level: f32,
    samples_in_state: usize,
}

impl Default for Adsr {
    fn default() -> Self {
        Self {
            attack: 0.01,
            decay: 0.1,
            sustain: 0.7,
            release: 0.3,
            state: EnvState::Idle,
            level: 0.0,
            samples_in_state: 0,
        }
    }
}

impl Adsr {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gate_on(&mut self) {
        self.state = EnvState::Attack;
        self.samples_in_state = 0;
    }

    pub fn gate_off(&mut self) {
        if self.state != EnvState::Idle {
            self.state = EnvState::Release;
            self.samples_in_state = 0;
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvState::Idle
    }

    /// Overwrite `out` with the per-sample envelope value.
    pub fn process(&mut self, out: &mut [f32]) {
        let _ = (out, &self.level, self.samples_in_state);
        todo!("port ADSR.render from synth/dsp/envelope.py")
    }

    pub fn reset(&mut self) {
        self.state = EnvState::Idle;
        self.level = 0.0;
        self.samples_in_state = 0;
    }
}
