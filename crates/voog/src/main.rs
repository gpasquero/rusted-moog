//! Rusted Moog — native application entry point. Wires the lock-free event queue
//! between the GUI/MIDI producers and the real-time audio consumer.
//!
//! The WebAssembly entry point lives in `lib.rs` under
//! `#[cfg(target_arch = "wasm32")]`; this binary is native-only.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    use std::sync::Arc;
    use voog::shared::SharedState;
    use voog::{audio, gui, midi};
    use voog_dsp::{patches::factory_presets, Event, Synth};

    // Multi-producer (GUI + MIDI) -> single-consumer (audio) event queue.
    let (tx, rx) = crossbeam_channel::bounded::<Event>(4096);
    let shared = Arc::new(SharedState::new());

    let synth = Synth::new();

    // Start audio; keep the stream alive for the whole program.
    let _stream = audio::start(synth, rx, shared.clone())?;

    // Start MIDI input (optional; keep the connection alive if present).
    let _midi_conn = midi::start(tx.clone(), 0);

    // Run the GUI on the main thread; blocks until the window is closed.
    gui::run(tx, shared, factory_presets()).map_err(|e| anyhow::anyhow!("GUI error: {e}"))?;

    Ok(())
}

// On wasm the binary target is not used (the library's `#[wasm_bindgen(start)]`
// is the real entry point); provide an empty `main` so the bin still compiles.
#[cfg(target_arch = "wasm32")]
fn main() {}
