//! A multitimbral channel: owns a patch and a voice allocator.
//! PORT OF `synth/engine/channel.py`.

use crate::arpeggiator::{ArpAction, ArpMode, Arpeggiator};
use crate::config::{BUFFER_SIZE, MAX_VOICES};
use crate::event::ParamId;
use crate::params::{GlideMode, LfoDest, NoiseType, Patch, Waveform};
use crate::voice_allocator::VoiceAllocator;

pub struct Channel {
    pub patch: Patch,
    pub allocator: VoiceAllocator,
    pub volume: f32,
    pub arp: Arpeggiator,
    arp_actions: Vec<ArpAction>,
}

impl Default for Channel {
    fn default() -> Self {
        Self::new()
    }
}

impl Channel {
    pub fn new() -> Self {
        let patch = Patch::default();
        let mut ch = Self {
            patch,
            allocator: VoiceAllocator::new(MAX_VOICES),
            volume: 1.0,
            arp: Arpeggiator::new(),
            arp_actions: Vec::with_capacity(8),
        };
        ch.reapply_patch();
        ch
    }

    fn reapply_patch(&mut self) {
        for v in &mut self.allocator.voices {
            v.apply_patch(&self.patch);
        }
    }

    pub fn set_patch(&mut self, patch: Patch) {
        self.patch = patch;
        self.reapply_patch();
    }

    pub fn note_on(&mut self, note: i32, velocity: u8) {
        if self.arp.enabled {
            // Route held notes into the arpeggiator; it drives the allocator.
            self.arp.note_on(note, velocity);
        } else {
            let idx = self.allocator.note_on(note, velocity);
            // Freshly stolen/allocated voice must carry the current patch.
            self.allocator.voices[idx].apply_patch(&self.patch);
        }
    }

    pub fn note_off(&mut self, note: i32) {
        if self.arp.enabled {
            self.arp.note_off(note);
        } else {
            self.allocator.note_off(note);
        }
    }

    pub fn all_notes_off(&mut self) {
        self.arp.reset(&mut self.arp_actions);
        self.arp_actions.clear();
        self.allocator.all_notes_off();
    }

    /// Update one continuous parameter and push it to all voices.
    pub fn set_param(&mut self, param: ParamId, value: f32) {
        self.patch.apply_param(param, value);
        self.reapply_patch();
    }

    pub fn set_osc_waveform(&mut self, osc: usize, waveform: Waveform) {
        if let Some(o) = self.patch.oscillators.get_mut(osc) {
            o.waveform = waveform;
            self.reapply_patch();
        }
    }

    pub fn set_noise_type(&mut self, noise_type: NoiseType) {
        self.patch.noise.noise_type = noise_type;
        self.reapply_patch();
    }

    pub fn set_lfo_waveform(&mut self, waveform: Waveform) {
        self.patch.lfo.waveform = waveform;
        self.reapply_patch();
    }

    pub fn set_lfo_dest(&mut self, dest: LfoDest) {
        self.patch.lfo.destination = dest;
        self.reapply_patch();
    }

    pub fn set_glide_mode(&mut self, mode: GlideMode) {
        self.patch.glide.mode = mode;
        self.reapply_patch();
    }

    // ── Arpeggiator ──
    pub fn set_arp_enabled(&mut self, on: bool) {
        if on == self.arp.enabled {
            return;
        }
        self.arp.enabled = on;
        // Toggling clears any hanging notes so nothing sticks.
        self.arp.reset(&mut self.arp_actions);
        self.arp_actions.clear();
        self.allocator.all_notes_off();
    }

    pub fn set_arp_mode(&mut self, mode: ArpMode) {
        self.arp.set_mode(mode);
    }

    pub fn set_arp_octaves(&mut self, octaves: u8) {
        self.arp.set_octaves(octaves);
    }

    pub fn set_arp_rate(&mut self, hz: f32) {
        self.arp.rate = hz;
    }

    pub fn set_arp_gate(&mut self, gate: f32) {
        self.arp.gate = gate;
    }

    pub fn set_arp_hold(&mut self, on: bool) {
        self.arp.set_hold(on);
    }

    /// Render all active voices, ADD into `out` (len <= BUFFER_SIZE), scaled by
    /// channel volume and the patch master volume.
    pub fn render_add(&mut self, out: &mut [f32], scratch: &mut [f32; BUFFER_SIZE]) {
        let n = out.len();

        // Advance the arpeggiator clock and apply its note events to the allocator.
        if self.arp.enabled {
            self.arp_actions.clear();
            self.arp.advance(n, &mut self.arp_actions);
            let mut i = 0;
            while i < self.arp_actions.len() {
                match self.arp_actions[i] {
                    ArpAction::NoteOn { note, velocity } => {
                        let idx = self.allocator.note_on(note, velocity);
                        self.allocator.voices[idx].apply_patch(&self.patch);
                    }
                    ArpAction::NoteOff { note } => self.allocator.note_off(note),
                }
                i += 1;
            }
        }

        let buf = &mut scratch[..n];
        buf.fill(0.0);
        for v in &mut self.allocator.voices {
            if v.active {
                v.render_add(buf);
            }
        }
        let gain = self.volume * self.patch.master_volume;
        for (o, &b) in out.iter_mut().zip(buf.iter()) {
            *o += b * gain;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_note() {
        let mut ch = Channel::new();
        ch.note_on(69, 100);
        let mut out = [0.0f32; 256];
        let mut scratch = [0.0f32; BUFFER_SIZE];
        ch.render_add(&mut out, &mut scratch);
        assert!(out.iter().any(|&s| s.abs() > 1e-4));
    }

    #[test]
    fn set_param_updates_patch_and_voices() {
        let mut ch = Channel::new();
        ch.set_param(ParamId::FilterCutoff, 1234.0);
        assert_eq!(ch.patch.filter.cutoff, 1234.0);
    }
}
