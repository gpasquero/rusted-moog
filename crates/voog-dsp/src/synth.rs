//! Master synth engine core. PORT OF `synth/engine/audio_engine.py` minus the
//! audio I/O (that lives in the `voog` binary crate's cpal callback).
//!
//! `render` is the real-time entry point: it drains no queue itself (the audio
//! layer applies [`Event`]s via [`Synth::apply_event`] before calling render),
//! mixes all channels, soft-clips, and tracks the peak level for the VU meter.

use crate::channel::Channel;
use crate::config::{BUFFER_SIZE, NUM_CHANNELS};
use crate::event::Event;

pub struct Synth {
    pub channels: Vec<Channel>,
    master_volume: f32,
    peak_level: f32,
    mix_scratch: [f32; BUFFER_SIZE],
    ch_scratch: [f32; BUFFER_SIZE],
}

impl Default for Synth {
    fn default() -> Self {
        Self::new()
    }
}

impl Synth {
    pub fn new() -> Self {
        Self {
            channels: (0..NUM_CHANNELS).map(|_| Channel::new()).collect(),
            master_volume: 0.8,
            peak_level: 0.0,
            mix_scratch: [0.0; BUFFER_SIZE],
            ch_scratch: [0.0; BUFFER_SIZE],
        }
    }

    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }

    pub fn set_master_volume(&mut self, v: f32) {
        self.master_volume = v.clamp(0.0, 1.0);
    }

    pub fn peak_level(&self) -> f32 {
        self.peak_level
    }

    pub fn active_voice_count(&self) -> usize {
        self.channels
            .iter()
            .map(|c| c.allocator.active_voice_count())
            .sum()
    }

    /// Apply one control event to the engine.
    pub fn apply_event(&mut self, ev: Event) {
        match ev {
            Event::MasterVolume(v) => self.set_master_volume(v),
            Event::NoteOn {
                channel,
                note,
                velocity,
            } => {
                if let Some(c) = self.channel_mut(channel) {
                    if velocity > 0 {
                        c.note_on(note, velocity);
                    } else {
                        c.note_off(note);
                    }
                }
            }
            Event::NoteOff { channel, note } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.note_off(note);
                }
            }
            Event::AllNotesOff { channel } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.all_notes_off();
                }
            }
            Event::SetParam {
                channel,
                param,
                value,
            } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.set_param(param, value);
                }
            }
            Event::SetOscWaveform {
                channel,
                osc,
                waveform,
            } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.set_osc_waveform(osc, waveform);
                }
            }
            Event::SetNoiseType {
                channel,
                noise_type,
            } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.set_noise_type(noise_type);
                }
            }
            Event::SetLfoWaveform { channel, waveform } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.set_lfo_waveform(waveform);
                }
            }
            Event::SetLfoDest { channel, dest } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.set_lfo_dest(dest);
                }
            }
            Event::SetGlideMode { channel, mode } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.set_glide_mode(mode);
                }
            }
            Event::LoadPatch { channel, patch } => {
                if let Some(c) = self.channel_mut(channel) {
                    c.set_patch(*patch);
                }
            }
        }
    }

    fn channel_mut(&mut self, channel: u8) -> Option<&mut Channel> {
        let idx = channel as usize;
        let idx = if idx >= self.channels.len() { 0 } else { idx };
        self.channels.get_mut(idx)
    }

    /// Render the full mix into `out` (any length; processed in BUFFER_SIZE
    /// chunks). Mono, soft-clipped with tanh. Updates the peak level.
    pub fn render(&mut self, out: &mut [f32]) {
        let mut peak = 0.0f32;
        for chunk in out.chunks_mut(BUFFER_SIZE) {
            let n = chunk.len();
            let mix = &mut self.mix_scratch[..n];
            mix.fill(0.0);
            for ch in &mut self.channels {
                ch.render_add(mix, &mut self.ch_scratch);
            }
            let mv = self.master_volume;
            for (o, &m) in chunk.iter_mut().zip(mix.iter()) {
                let s = (m * mv).tanh(); // soft clip
                *o = s;
                let a = s.abs();
                if a > peak {
                    peak = a;
                }
            }
        }
        self.peak_level = peak;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::Waveform;

    #[test]
    fn silent_when_idle() {
        let mut s = Synth::new();
        let mut out = [0.0f32; 512];
        s.render(&mut out);
        assert!(out.iter().all(|&x| x == 0.0));
        assert_eq!(s.peak_level(), 0.0);
    }

    #[test]
    fn note_on_makes_sound_and_peaks() {
        let mut s = Synth::new();
        s.apply_event(Event::NoteOn {
            channel: 0,
            note: 69,
            velocity: 110,
        });
        let mut out = [0.0f32; 1024];
        s.render(&mut out);
        assert!(out.iter().any(|&x| x.abs() > 1e-3));
        assert!(s.peak_level() > 0.0);
        // Soft clip keeps everything in [-1, 1].
        assert!(out.iter().all(|&x| x.abs() <= 1.0));
        assert_eq!(s.active_voice_count(), 1);
    }

    #[test]
    fn arbitrary_length_render_is_chunked_safely() {
        let mut s = Synth::new();
        s.apply_event(Event::NoteOn {
            channel: 1,
            note: 60,
            velocity: 100,
        });
        // Non-multiple of BUFFER_SIZE exercises the chunk remainder path.
        let mut out = [0.0f32; 700];
        s.render(&mut out);
        assert!(out.iter().all(|&x| x.is_finite() && x.abs() <= 1.0));
    }

    #[test]
    fn events_route_to_channels() {
        let mut s = Synth::new();
        s.apply_event(Event::SetOscWaveform {
            channel: 0,
            osc: 0,
            waveform: Waveform::Square,
        });
        assert_eq!(
            s.channels[0].patch.oscillators[0].waveform,
            Waveform::Square
        );
        s.apply_event(Event::MasterVolume(0.5));
        assert_eq!(s.master_volume(), 0.5);
    }
}
