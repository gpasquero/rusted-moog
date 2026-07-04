//! Polyphonic voice allocator with oldest-note voice stealing.
//! PORT OF `synth/engine/voice_allocator.py`.

use crate::config::MAX_VOICES;
use crate::voice::Voice;

pub struct VoiceAllocator {
    pub voices: Vec<Voice>,
    age_counter: u64,
    voice_ages: Vec<u64>,
    held_notes: Vec<i32>,
}

impl Default for VoiceAllocator {
    fn default() -> Self {
        Self::new(MAX_VOICES)
    }
}

impl VoiceAllocator {
    pub fn new(max_voices: usize) -> Self {
        Self {
            voices: (0..max_voices).map(|_| Voice::new()).collect(),
            age_counter: 0,
            voice_ages: vec![0; max_voices],
            held_notes: Vec::with_capacity(max_voices * 2),
        }
    }

    /// Allocate/steal a voice for `note` and gate it on. Returns its index.
    pub fn note_on(&mut self, note: i32, velocity: u8) -> usize {
        let legato = !self.held_notes.is_empty();
        self.held_notes.push(note);

        // 1. Re-use a voice already playing this note.
        // 2. Otherwise a free voice.
        // 3. Otherwise steal the oldest.
        let idx = self
            .voices
            .iter()
            .position(|v| v.note == note && v.active)
            .or_else(|| self.voices.iter().position(|v| !v.active))
            .unwrap_or_else(|| {
                // oldest = smallest age
                let mut oldest = 0usize;
                for i in 1..self.voice_ages.len() {
                    if self.voice_ages[i] < self.voice_ages[oldest] {
                        oldest = i;
                    }
                }
                self.voices[oldest].reset();
                oldest
            });

        self.voices[idx].note_on(note, velocity, legato);
        self.age_counter += 1;
        self.voice_ages[idx] = self.age_counter;
        idx
    }

    pub fn note_off(&mut self, note: i32) {
        if let Some(pos) = self.held_notes.iter().position(|&n| n == note) {
            self.held_notes.remove(pos);
        }
        for v in &mut self.voices {
            if v.note == note && v.active {
                v.note_off();
            }
        }
    }

    pub fn all_notes_off(&mut self) {
        self.held_notes.clear();
        for v in &mut self.voices {
            if v.active {
                v.note_off();
            }
        }
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocates_free_voices_then_reuses_note() {
        let mut a = VoiceAllocator::new(4);
        let i0 = a.note_on(60, 100);
        let i1 = a.note_on(64, 100);
        assert_ne!(i0, i1);
        assert_eq!(a.active_voice_count(), 2);
        // Same note re-uses the same voice.
        let i0b = a.note_on(60, 100);
        assert_eq!(i0, i0b);
    }

    #[test]
    fn steals_oldest_when_full() {
        let mut a = VoiceAllocator::new(2);
        let first = a.note_on(60, 100);
        let _second = a.note_on(62, 100);
        // Third note with all busy -> steal the oldest (the first).
        let third = a.note_on(64, 100);
        assert_eq!(third, first, "should steal the oldest voice");
        assert_eq!(a.voices[third].note, 64);
    }

    #[test]
    fn note_off_gates_release() {
        let mut a = VoiceAllocator::new(4);
        a.note_on(60, 100);
        a.note_off(60);
        // Still 'active' (releasing) until the envelope finishes, but held set is empty.
        a.all_notes_off();
        assert_eq!(a.active_voice_count(), 1); // releasing, not yet silent
    }
}
