//! cpal audio output. Drains the event queue then renders each block on the
//! real-time thread.
//!
//! Port of the callback in `synth/engine/audio_engine.py`, except master volume,
//! tanh soft-clip and peak metering all live inside [`Synth::render`] — so the
//! callback only drains pending events, renders a mono block, fans it out across
//! the device's interleaved channels, and publishes the meters.

use crate::shared::{EventReceiver, SharedState};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use voog_dsp::Synth;

/// Generous upper bound for the mono render scratch. Blocks larger than this are
/// clamped rather than allocated, keeping the callback allocation-free.
const MAX_BLOCK: usize = 8192;

/// Build and start the output stream. The returned `Stream` must be kept alive
/// for audio to keep flowing.
pub fn start(
    synth: Synth,
    rx: EventReceiver,
    shared: Arc<SharedState>,
) -> anyhow::Result<cpal::Stream> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow::anyhow!("no default output device"))?;
    let config = device.default_output_config()?;

    // We only support f32 output.
    if config.sample_format() != cpal::SampleFormat::F32 {
        anyhow::bail!(
            "unsupported output sample format: {:?} (only f32 is supported)",
            config.sample_format()
        );
    }

    let channels = config.channels() as usize;
    #[allow(unused_mut)]
    let mut stream_config: cpal::StreamConfig = config.into();

    // On the web, cpal drives audio from a main-thread timer that competes with
    // egui's canvas rendering. A larger fixed buffer gives the scheduler much
    // more lead time, so main-thread jank no longer causes dropouts (trading a
    // little latency for glitch-free playback). Native keeps the device default.
    #[cfg(target_arch = "wasm32")]
    {
        stream_config.buffer_size = cpal::BufferSize::Fixed(4096);
    }

    // Move the engine into the callback. Pre-allocate the mono scratch buffer
    // OUTSIDE the callback so the real-time thread never allocates.
    let mut synth = synth;
    let mut scratch: Vec<f32> = Vec::with_capacity(MAX_BLOCK);
    scratch.resize(MAX_BLOCK, 0.0);

    let data_callback = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        // Drain ALL pending control events before rendering this block.
        while let Ok(ev) = rx.try_recv() {
            synth.apply_event(ev);
        }

        let channels = channels.max(1);
        let mut frames = data.len() / channels;
        // Never allocate: clamp to the pre-allocated scratch length.
        if frames > scratch.len() {
            frames = scratch.len();
        }

        let mono = &mut scratch[..frames];
        synth.render(mono);

        // Fan the mono signal out into the interleaved output frames.
        for (frame, &s) in data.chunks_mut(channels).zip(mono.iter()) {
            for out in frame.iter_mut() {
                *out = s;
            }
        }
        // If the device asked for more frames than we clamped to, silence the
        // remainder rather than leaving stale data.
        for out in data[frames * channels..].iter_mut() {
            *out = 0.0;
        }

        // Publish meters for the GUI (lock-free atomics).
        shared.set_peak(synth.peak_level());
        shared.set_voices(synth.active_voice_count());
    };

    let err_callback = |_err: cpal::StreamError| {
        // Real-time safe: do nothing heavy here.
    };

    let stream = device.build_output_stream(&stream_config, data_callback, err_callback, None)?;
    stream.play()?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use voog_dsp::{Event, Synth};

    /// Pure-logic smoke test: no audio device required. Builds a `Synth`,
    /// triggers a note, and renders directly, asserting finite output.
    #[test]
    fn render_produces_finite_output() {
        let mut synth = Synth::new();
        synth.apply_event(Event::NoteOn {
            channel: 0,
            note: 69,
            velocity: 110,
        });

        let mut buf = [0.0f32; 512];
        synth.render(&mut buf);

        assert!(buf.iter().all(|&x| x.is_finite()));
        assert!(buf.iter().any(|&x| x.abs() > 0.0));
        assert!(synth.peak_level() > 0.0);
        assert_eq!(synth.active_voice_count(), 1);
    }
}
