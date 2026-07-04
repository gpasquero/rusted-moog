//! Jupiter-8-style arpeggiator.
//!
//! Sits between note input and the voice allocator: while enabled, physically
//! held notes form a chord that is played back one note at a time on an
//! internal clock. Modes UP / DOWN / UP&DOWN / RANDOM, an octave range of 1–4,
//! a free-running rate (Hz), a per-step GATE length, and a HOLD (latch) that
//! keeps the arpeggio running after the keys are released.
//!
//! The engine is sample-accurate at block granularity: `advance()` is called
//! once per audio block and emits [`ArpAction`]s (note on/off) for the allocator
//! to apply. It never allocates in the hot path (actions go into a caller-owned,
//! reused `Vec`).

use crate::config::SAMPLE_RATE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpMode {
    Up,
    Down,
    UpDown,
    Random,
}

/// A note event produced by the arpeggiator for the allocator to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpAction {
    NoteOn { note: i32, velocity: u8 },
    NoteOff { note: i32 },
}

pub struct Arpeggiator {
    pub enabled: bool,
    pub mode: ArpMode,
    pub octaves: u8, // 1..=4
    pub rate: f32,   // Hz
    pub gate: f32,   // 0..1 fraction of the step the note sounds
    pub hold: bool,

    held: Vec<i32>, // physically held notes
    set: Vec<i32>,  // sorted unique base notes being arpeggiated
    seq: Vec<i32>,  // expanded, mode-ordered playback sequence
    step: usize,
    phase: f32,           // samples elapsed in the current step
    current: Option<i32>, // currently sounding arp note
    note_on: bool,        // is `current` gated on
    velocity: u8,
    rng: u32,
}

impl Default for Arpeggiator {
    fn default() -> Self {
        Self::new()
    }
}

impl Arpeggiator {
    pub fn new() -> Self {
        Self {
            enabled: false,
            mode: ArpMode::Up,
            octaves: 1,
            rate: 8.0,
            gate: 0.5,
            hold: false,
            held: Vec::with_capacity(16),
            set: Vec::with_capacity(16),
            seq: Vec::with_capacity(64),
            step: 0,
            phase: 0.0,
            current: None,
            note_on: false,
            velocity: 100,
            rng: 0x9E37_79B9,
        }
    }

    /// Register a physically pressed key.
    pub fn note_on(&mut self, note: i32, velocity: u8) {
        self.velocity = velocity.max(1);
        let was_empty = self.held.is_empty();
        if !self.held.contains(&note) {
            self.held.push(note);
        }
        // With HOLD, the first key pressed after a full release starts a brand
        // new chord (clears the latched set).
        if self.hold && was_empty {
            self.set.clear();
        }
        if !self.set.contains(&note) {
            self.set.push(note);
            self.set.sort_unstable();
        }
        self.rebuild_seq();
    }

    /// Register a physically released key.
    pub fn note_off(&mut self, note: i32) {
        self.held.retain(|&n| n != note);
        if !self.hold {
            self.set.retain(|&n| n != note);
            self.rebuild_seq();
        }
        // With HOLD the note stays latched in `set`.
    }

    /// Change the direction mode (rebuilds the sequence in place).
    pub fn set_mode(&mut self, mode: ArpMode) {
        self.mode = mode;
        self.rebuild_seq();
    }

    /// Change the octave range 1..=4 (rebuilds the sequence in place).
    pub fn set_octaves(&mut self, octaves: u8) {
        self.octaves = octaves.clamp(1, 4);
        self.rebuild_seq();
    }

    /// Toggle HOLD. Turning it off drops any latched notes no longer held.
    pub fn set_hold(&mut self, on: bool) {
        self.hold = on;
        if !on {
            let held = &self.held;
            self.set.retain(|n| held.contains(n));
            self.rebuild_seq();
        }
    }

    /// Rebuild the mode-ordered, octave-expanded playback sequence.
    fn rebuild_seq(&mut self) {
        self.seq.clear();
        if self.set.is_empty() {
            self.step = 0;
            return;
        }
        let octaves = self.octaves.clamp(1, 4) as i32;
        // Ascending across octaves.
        let mut asc: Vec<i32> = Vec::with_capacity(self.set.len() * octaves as usize);
        for o in 0..octaves {
            for &n in &self.set {
                asc.push(n + 12 * o);
            }
        }
        match self.mode {
            ArpMode::Up | ArpMode::Random => self.seq = asc,
            ArpMode::Down => {
                asc.reverse();
                self.seq = asc;
            }
            ArpMode::UpDown => {
                self.seq = asc.clone();
                // Append the descending middle (excluding both endpoints) so the
                // top and bottom notes aren't repeated back-to-back.
                if asc.len() > 2 {
                    for &n in asc[1..asc.len() - 1].iter().rev() {
                        self.seq.push(n);
                    }
                }
            }
        }
        if !self.seq.is_empty() {
            self.step %= self.seq.len();
        }
    }

    #[inline]
    fn rng_next(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        if x == 0 {
            x = 0x9E37_79B9;
        }
        self.rng = x;
        x
    }

    fn next_note(&mut self) -> i32 {
        match self.mode {
            ArpMode::Random => {
                let i = (self.rng_next() as usize) % self.seq.len();
                self.seq[i]
            }
            _ => {
                let n = self.seq[self.step % self.seq.len()];
                self.step = (self.step + 1) % self.seq.len();
                n
            }
        }
    }

    /// Advance the arpeggiator clock by `n` samples, pushing note on/off actions
    /// into `actions` (which the caller clears/reuses). Safe to call every block;
    /// the rate is clamped so at most one step boundary falls inside one block.
    pub fn advance(&mut self, n: usize, actions: &mut Vec<ArpAction>) {
        // If we're not producing notes, make sure nothing is left hanging.
        if !self.enabled || self.seq.is_empty() {
            if let Some(c) = self.current.take() {
                actions.push(ArpAction::NoteOff { note: c });
            }
            self.note_on = false;
            self.phase = 0.0;
            return;
        }

        // Trigger the very first note the moment a chord appears.
        if self.current.is_none() {
            let note = self.next_note();
            actions.push(ArpAction::NoteOn {
                note,
                velocity: self.velocity,
            });
            self.current = Some(note);
            self.note_on = true;
            self.phase = 0.0;
            return;
        }

        let rate = self.rate.clamp(0.5, 25.0);
        let period = (SAMPLE_RATE / rate).max(2.0);
        let gate_samples = period * self.gate.clamp(0.02, 1.0);

        let old = self.phase;
        let new = old + n as f32;

        // Gate-off within the current step.
        if self.note_on && old < gate_samples && new >= gate_samples {
            if let Some(c) = self.current {
                actions.push(ArpAction::NoteOff { note: c });
            }
            self.note_on = false;
        }

        if new >= period {
            // Step boundary: end the current note (if still on) and start the next.
            self.phase = new - period;
            if self.note_on {
                if let Some(c) = self.current {
                    actions.push(ArpAction::NoteOff { note: c });
                }
                self.note_on = false;
            }
            let note = self.next_note();
            actions.push(ArpAction::NoteOn {
                note,
                velocity: self.velocity,
            });
            self.current = Some(note);
            self.note_on = true;
        } else {
            self.phase = new;
        }
    }

    /// Silence and forget everything (e.g. when toggling the arp or all-notes-off).
    pub fn reset(&mut self, actions: &mut Vec<ArpAction>) {
        if let Some(c) = self.current.take() {
            actions.push(ArpAction::NoteOff { note: c });
        }
        self.held.clear();
        self.set.clear();
        self.seq.clear();
        self.step = 0;
        self.phase = 0.0;
        self.note_on = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_notes(actions: &[ArpAction]) -> Vec<i32> {
        actions
            .iter()
            .filter_map(|a| match a {
                ArpAction::NoteOn { note, .. } => Some(*note),
                _ => None,
            })
            .collect()
    }

    /// Run the arp for `steps` clock steps and collect the note-on order.
    fn play(arp: &mut Arpeggiator, steps: usize) -> Vec<i32> {
        let period = (SAMPLE_RATE / arp.rate) as usize;
        let mut acts = Vec::new();
        let mut out = Vec::new();
        // First block triggers the initial note.
        arp.advance(64, &mut acts);
        out.extend(on_notes(&acts));
        for _ in 0..steps {
            acts.clear();
            // advance a whole step in one block (period > block is guaranteed,
            // but for the test we can jump a full period at once)
            arp.advance(period + 4, &mut acts);
            out.extend(on_notes(&acts));
        }
        out
    }

    #[test]
    fn disabled_produces_nothing() {
        let mut arp = Arpeggiator::new();
        arp.note_on(60, 100);
        let mut acts = Vec::new();
        arp.advance(512, &mut acts);
        assert!(acts.is_empty());
    }

    #[test]
    fn up_mode_cycles_ascending() {
        let mut arp = Arpeggiator::new();
        arp.enabled = true;
        arp.mode = ArpMode::Up;
        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_on(67, 100);
        let seq = play(&mut arp, 5);
        // 60,64,67 repeating
        assert_eq!(&seq[0..3], &[60, 64, 67]);
        assert_eq!(seq[3], 60);
    }

    #[test]
    fn down_mode_cycles_descending() {
        let mut arp = Arpeggiator::new();
        arp.enabled = true;
        arp.mode = ArpMode::Down;
        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_on(67, 100);
        let seq = play(&mut arp, 3);
        assert_eq!(&seq[0..3], &[67, 64, 60]);
    }

    #[test]
    fn octave_range_expands_upward() {
        let mut arp = Arpeggiator::new();
        arp.enabled = true;
        arp.mode = ArpMode::Up;
        arp.octaves = 2;
        arp.note_on(60, 100);
        arp.note_on(64, 100);
        let seq = play(&mut arp, 4);
        // 60,64,72,76 repeating
        assert_eq!(&seq[0..4], &[60, 64, 72, 76]);
    }

    #[test]
    fn updown_bounces() {
        let mut arp = Arpeggiator::new();
        arp.enabled = true;
        arp.mode = ArpMode::UpDown;
        arp.note_on(60, 100);
        arp.note_on(64, 100);
        arp.note_on(67, 100);
        let seq = play(&mut arp, 5);
        // 60,64,67,64,(60,64...) — middle bounces without repeating endpoints
        assert_eq!(&seq[0..4], &[60, 64, 67, 64]);
        assert_eq!(seq[4], 60);
    }

    #[test]
    fn hold_latches_after_release() {
        let mut arp = Arpeggiator::new();
        arp.enabled = true;
        arp.hold = true;
        arp.note_on(60, 100);
        arp.note_on(63, 100);
        // release both keys — with HOLD the set stays
        arp.note_off(60);
        arp.note_off(63);
        let seq = play(&mut arp, 3);
        assert!(seq.contains(&60) && seq.contains(&63));
    }

    #[test]
    fn gate_releases_before_step_end() {
        let mut arp = Arpeggiator::new();
        arp.enabled = true;
        arp.rate = 8.0;
        arp.gate = 0.5;
        arp.note_on(60, 100);
        let period = (SAMPLE_RATE / arp.rate) as usize;
        let mut acts = Vec::new();
        arp.advance(64, &mut acts); // first note on
        acts.clear();
        // advance to just past the gate point (60% of the way) — should note-off
        arp.advance(period * 6 / 10, &mut acts);
        assert!(acts.iter().any(|a| matches!(a, ArpAction::NoteOff { .. })));
    }

    #[test]
    fn no_notes_is_silent() {
        let mut arp = Arpeggiator::new();
        arp.enabled = true;
        let mut acts = Vec::new();
        arp.advance(1024, &mut acts);
        assert!(acts.is_empty());
    }
}
