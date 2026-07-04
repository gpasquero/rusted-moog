//! State shared between the audio thread (writer) and the GUI thread (reader),
//! plus the event-channel type aliases. Lock-free via atomics.

use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};
use voog_dsp::Event;

/// Multi-producer (GUI + MIDI) -> single-consumer (audio) event channel.
pub type EventSender = crossbeam_channel::Sender<Event>;
pub type EventReceiver = crossbeam_channel::Receiver<Event>;

/// Realtime meters published by the audio thread for the GUI to read.
#[derive(Default)]
pub struct SharedState {
    peak_bits: AtomicU32,
    voices: AtomicUsize,
}

impl SharedState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_peak(&self, v: f32) {
        self.peak_bits.store(v.to_bits(), Ordering::Relaxed);
    }

    pub fn peak(&self) -> f32 {
        f32::from_bits(self.peak_bits.load(Ordering::Relaxed))
    }

    pub fn set_voices(&self, n: usize) {
        self.voices.store(n, Ordering::Relaxed);
    }

    pub fn voices(&self) -> usize {
        self.voices.load(Ordering::Relaxed)
    }
}
