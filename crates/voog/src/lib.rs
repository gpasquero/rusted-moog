//! Rusted Moog — shared library crate.
//!
//! The same modules back two front-ends:
//! - the native desktop binary (`src/main.rs`), and
//! - the WebAssembly build bundled by `trunk` (the `#[wasm_bindgen(start)]`
//!   entry point below), which runs the *identical* egui/eframe `App` in the
//!   browser on an `HtmlCanvasElement`.

pub mod audio;
pub mod gui;
pub mod shared;

// MIDI input is desktop-only (`midir` does not build on wasm).
#[cfg(not(target_arch = "wasm32"))]
pub mod midi;

// ── WebAssembly entry point ─────────────────────────────────────────────────
#[cfg(target_arch = "wasm32")]
mod web {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    /// Browser entry point. Called automatically once the wasm module loads.
    ///
    /// Installs the panic hook (so Rust panics surface in the JS console),
    /// grabs the `<canvas id="the_canvas_id">` element, and boots the real
    /// eframe `App` on it. The `App` self-initializes the synth engine and the
    /// (best-effort) Web Audio output stream.
    #[wasm_bindgen(start)]
    pub async fn start() -> Result<(), JsValue> {
        console_error_panic_hook::set_once();

        let document = web_sys::window()
            .ok_or_else(|| JsValue::from_str("no window"))?
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .ok_or_else(|| JsValue::from_str("canvas `the_canvas_id` not found"))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("element is not a canvas"))?;

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(crate::gui::App::new_web(cc)))),
            )
            .await?;
        Ok(())
    }
}
