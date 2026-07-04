//! voog-dsp — pure real-time-safe DSP core for the Rusted Moog synthesizer.
//!
//! Ported from the Python `synth/dsp/*` and `synth/config.py`. No audio I/O,
//! no GUI. Every module is independently unit-testable.
//!
//! ## Buffer / ownership convention (IMPORTANT for all modules)
//! - Audio is `f32`, mono, processed in blocks.
//! - Generators that MIX into a signal expose `process_add(&mut self, out: &mut [f32], ...)`
//!   which ADDS their (already level-scaled) contribution into `out`.
//! - Processors that transform a signal expose `process(&mut self, io: &mut [f32], ...)`
//!   operating in place.
//! - Control sources (envelope, LFO, glide) expose `process(&mut self, out: &mut [f32])`
//!   which OVERWRITES `out` with their per-sample signal.
//! - No heap allocation in these hot-path methods.

pub mod config;
pub mod params;
pub mod wavetables;

pub mod envelope;
pub mod filter;
pub mod glide;
pub mod lfo;
pub mod noise;
pub mod oscillator;

// Engine layer (pure logic, no audio I/O — fully unit-testable).
pub mod arpeggiator;
pub mod channel;
pub mod effects;
pub mod event;
pub mod patches;
pub mod synth;
pub mod voice;
pub mod voice_allocator;

pub use config::{midi_to_freq, SAMPLE_RATE};
pub use envelope::Adsr;
pub use filter::MoogFilter;
pub use glide::Glide;
pub use lfo::Lfo;
pub use noise::NoiseGenerator;
pub use oscillator::Oscillator;
pub use params::*;

pub use arpeggiator::{ArpMode, Arpeggiator};
pub use channel::Channel;
pub use effects::{Chorus, Delay};
pub use event::{Event, ParamId};
pub use synth::Synth;
pub use voice::Voice;
pub use voice_allocator::VoiceAllocator;
