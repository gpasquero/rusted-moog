//! Rusted Moog — application entry point. Wires the lock-free event queue
//! between the GUI/MIDI producers and the real-time audio consumer.

mod audio;
mod gui;
mod midi;
mod shared;

use std::sync::Arc;
use voog_dsp::{patches::factory_presets, Event, Synth};

fn main() -> anyhow::Result<()> {
    // Multi-producer (GUI + MIDI) -> single-consumer (audio) event queue.
    let (tx, rx) = crossbeam_channel::bounded::<Event>(4096);
    let shared = Arc::new(shared::SharedState::new());

    let synth = Synth::new();

    // Start audio; keep the stream alive for the whole program.
    let _stream = audio::start(synth, rx, shared.clone())?;

    // Start MIDI input (optional; keep the connection alive if present).
    let _midi_conn = midi::start(tx.clone(), 0);

    // Run the GUI on the main thread; blocks until the window is closed.
    gui::run(tx, shared, factory_presets()).map_err(|e| anyhow::anyhow!("GUI error: {e}"))?;

    Ok(())
}
