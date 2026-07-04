# Rusted Moog

A **virtual analog synthesizer** written in Rust — a from-scratch port of the
Python [VOOG](https://github.com/gpasquero/voog) synth, rebuilt for real-time,
glitch-free audio with a modern GUI.

> Why the rewrite? The Python engine suffered from audio dropouts caused by
> garbage-collection pauses and GIL contention inside the audio callback. Rust
> gives us deterministic, allocation-free real-time audio: no GC, no glitches.

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│  crate: voog  (binary)                                         │
│                                                                │
│   egui GUI  ──params/notes──►  lock-free channel  ──►  audio   │
│   (main thread)                (ringbuf)              callback  │
│        ▲                                            (cpal RT)   │
│        └──────────── VU / voice count ◄────────────────┘       │
│                                                                │
│   midir  ──MIDI events──►  same lock-free channel              │
└──────────────────────────────────────────────────────────────┘
              │ depends on
              ▼
┌──────────────────────────────────────────────────────────────┐
│  crate: voog-dsp  (library, no I/O, fully unit-tested)         │
│   oscillator · moog filter · adsr · lfo · noise · glide        │
│   wavetables · params (patch model) · config                   │
└──────────────────────────────────────────────────────────────┘
```

**Real-time discipline:** the audio callback never allocates and never locks.
The GUI and MIDI threads communicate with the engine through a lock-free
ring buffer of events (note on/off, parameter changes). This is what eliminates
the dropouts the Python version had.

## Signal chain (per voice)

3 oscillators (+ noise) → Moog ladder filter (24 dB/oct) → amp VCA
with dual ADSR envelopes (amp + filter), an LFO (filter/pitch/amp), and
glide/portamento. 4 multitimbral channels × 8-voice polyphony.

## Feature parity target (with Python VOOG)

- [x] `voog-dsp` core: oscillator, filter, envelope, LFO, noise, glide
- [ ] Voice / polyphony / voice-stealing allocator
- [ ] Multitimbral channels + master engine
- [ ] cpal audio output (lock-free event queue)
- [ ] MIDI input (midir) with the same CC map
- [ ] Patch model + built-in presets + save/load (JSON, compatible with VOOG)
- [ ] egui GUI: rotary knobs, VU meter, virtual keyboard, dark Moog theme

## Build & run

```bash
cargo test          # run the DSP unit-test suite
cargo run --release # launch the synth
```

Requires a stable Rust toolchain (`rustup`, edition 2021).

## Layout

```
crates/
  voog-dsp/   # pure DSP, no I/O — the testable heart
  voog/       # engine + cpal audio + midir + egui GUI  (added in later phases)
```

## License

MIT
