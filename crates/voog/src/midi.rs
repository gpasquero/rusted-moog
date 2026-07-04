//! MIDI input via `midir`. Parses raw MIDI bytes into `Event`s and forwards
//! them to the engine over the lock-free event channel.
//!
//! Behaviour is ported from the Python reference:
//! - `synth/midi/midi_input.py`  (note-on/off, control-change parsing)
//! - `synth/midi/midi_router.py` (MIDI channel -> synth channel routing)
//! - `synth/midi/cc_map.py`      (CC number -> parameter + scaling range)

use crate::shared::EventSender;
use midir::{Ignore, MidiInput, MidiInputConnection};
use voog_dsp::{Event, ParamId};

/// Number of multitimbral channels the engine exposes (channels `0..=3`).
/// Matches `NUM_CHANNELS` from the Python `synth.config`.
const NUM_CHANNELS: u8 = 4;

/// Where a mapped continuous CC should be delivered.
///
/// Most CCs address a typed [`ParamId`], but the Python `cc_map` also maps
/// CC 7 to `master_volume`, which the Rust engine exposes as the channel-less
/// [`Event::MasterVolume`] variant instead of a [`ParamId`].
#[derive(Debug, Clone, Copy, PartialEq)]
enum CcTarget {
    Param(ParamId),
    MasterVolume,
}

/// CC number -> (target, min, max), ported faithfully from `cc_map.py`.
///
/// The scaled value is `min + (cc_value / 127) * (max - min)`, exactly as the
/// Python engine computes it.
fn cc_target(cc: u8) -> Option<(CcTarget, f32, f32)> {
    let (target, min, max) = match cc {
        1 => (CcTarget::Param(ParamId::LfoDepth), 0.0, 1.0), // Mod wheel -> LFO depth
        7 => (CcTarget::MasterVolume, 0.0, 1.0),             // Channel / master volume
        71 => (CcTarget::Param(ParamId::FilterResonance), 0.0, 1.0), // Resonance
        74 => (CcTarget::Param(ParamId::FilterCutoff), 20.0, 20000.0), // Cutoff (brightness)
        73 => (CcTarget::Param(ParamId::AmpAttack), 0.001, 2.0), // Amp attack time
        75 => (CcTarget::Param(ParamId::AmpDecay), 0.001, 2.0), // Amp decay time
        72 => (CcTarget::Param(ParamId::AmpRelease), 0.001, 3.0), // Amp release time
        76 => (CcTarget::Param(ParamId::LfoRate), 0.1, 20.0), // LFO rate (vibrato rate)
        77 => (CcTarget::Param(ParamId::FilterEnvAmount), 0.0, 48.0), // Filter envelope amount
        78 => (CcTarget::Param(ParamId::FilterAttack), 0.001, 2.0), // Filter attack
        79 => (CcTarget::Param(ParamId::FilterDecay), 0.001, 2.0), // Filter decay
        _ => return None,
    };
    Some((target, min, max))
}

/// Scale a 0..=127 CC value into `[min, max]`, matching `cc_map.py`.
fn scale_cc(value: u8, min: f32, max: f32) -> f32 {
    min + (value as f32 / 127.0) * (max - min)
}

/// Route an incoming MIDI channel to a synth channel.
///
/// The Python `MidiRouter` uses `midi_channel % NUM_CHANNELS`; here we treat
/// any channel `>= NUM_CHANNELS` as falling back to `default_channel`
/// (which is `0` in normal operation), keeping everything within `0..=3`.
fn route_channel(midi_channel: u8, default_channel: u8) -> u8 {
    if midi_channel < NUM_CHANNELS {
        midi_channel
    } else {
        default_channel
    }
}

/// Parse a raw MIDI message into an [`Event`].
///
/// Returns `None` for messages we do not handle (or malformed/short ones).
/// `default_channel` is used when the message's own channel is out of range.
fn parse_message(bytes: &[u8], default_channel: u8) -> Option<Event> {
    if bytes.is_empty() {
        return None;
    }

    let status = bytes[0] & 0xF0;
    let midi_channel = bytes[0] & 0x0F;
    let channel = route_channel(midi_channel, default_channel);

    match status {
        // Note on
        0x90 => {
            let note = *bytes.get(1)? as i32;
            let velocity = *bytes.get(2)?;
            if velocity > 0 {
                Some(Event::NoteOn {
                    channel,
                    note,
                    velocity,
                })
            } else {
                // Velocity 0 is a note-off by convention.
                Some(Event::NoteOff { channel, note })
            }
        }
        // Note off
        0x80 => {
            let note = *bytes.get(1)? as i32;
            Some(Event::NoteOff { channel, note })
        }
        // Control change
        0xB0 => {
            let control = *bytes.get(1)?;
            let value = *bytes.get(2)?;
            // All Sound Off (120) / All Notes Off (123).
            if control == 120 || control == 123 {
                return Some(Event::AllNotesOff { channel });
            }
            let (target, min, max) = cc_target(control)?;
            let scaled = scale_cc(value, min, max);
            match target {
                CcTarget::Param(param) => Some(Event::SetParam {
                    channel,
                    param,
                    value: scaled,
                }),
                CcTarget::MasterVolume => Some(Event::MasterVolume(scaled)),
            }
        }
        _ => None,
    }
}

/// Connect to the first available MIDI input. The returned connection must be
/// kept alive. Returns `None` if no device / not available (graceful fallback).
pub fn start(tx: EventSender, channel: u8) -> Option<MidiInputConnection<()>> {
    let mut midi_in = MidiInput::new("voog-midi-in").ok()?;
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    let port = ports.first()?;
    let port_name = midi_in
        .port_name(port)
        .unwrap_or_else(|_| "voog-midi-in".to_string());

    let connection = midi_in
        .connect(
            port,
            &port_name,
            move |_timestamp, bytes, _| {
                if let Some(event) = parse_message(bytes, channel) {
                    // Ignore send errors: if the audio thread is gone there is
                    // nothing useful to do from the MIDI callback.
                    let _ = tx.send(event);
                }
            },
            (),
        )
        .ok()?;

    Some(connection)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_on_with_velocity() {
        // Channel 2, note 60, velocity 100.
        let ev = parse_message(&[0x92, 60, 100], 0).unwrap();
        match ev {
            Event::NoteOn {
                channel,
                note,
                velocity,
            } => {
                assert_eq!(channel, 2);
                assert_eq!(note, 60);
                assert_eq!(velocity, 100);
            }
            other => panic!("expected NoteOn, got {other:?}"),
        }
    }

    #[test]
    fn note_on_zero_velocity_is_note_off() {
        let ev = parse_message(&[0x90, 64, 0], 0).unwrap();
        match ev {
            Event::NoteOff { channel, note } => {
                assert_eq!(channel, 0);
                assert_eq!(note, 64);
            }
            other => panic!("expected NoteOff, got {other:?}"),
        }
    }

    #[test]
    fn note_off_message() {
        let ev = parse_message(&[0x81, 48, 40], 0).unwrap();
        match ev {
            Event::NoteOff { channel, note } => {
                assert_eq!(channel, 1);
                assert_eq!(note, 48);
            }
            other => panic!("expected NoteOff, got {other:?}"),
        }
    }

    #[test]
    fn channel_out_of_range_falls_back_to_default() {
        // MIDI channel 5 (>= NUM_CHANNELS) routes to the default channel.
        let ev = parse_message(&[0x95, 60, 100], 0).unwrap();
        match ev {
            Event::NoteOn { channel, .. } => assert_eq!(channel, 0),
            other => panic!("expected NoteOn, got {other:?}"),
        }
    }

    #[test]
    fn cc_cutoff_maps_and_scales() {
        // CC 74 (filter cutoff) at max value -> 20000 Hz.
        let ev = parse_message(&[0xB0, 74, 127], 0).unwrap();
        match ev {
            Event::SetParam {
                channel,
                param,
                value,
            } => {
                assert_eq!(channel, 0);
                assert_eq!(param, ParamId::FilterCutoff);
                assert!((value - 20000.0).abs() < 1e-3, "value = {value}");
            }
            other => panic!("expected SetParam, got {other:?}"),
        }

        // Minimum value -> 20 Hz.
        let ev = parse_message(&[0xB0, 74, 0], 0).unwrap();
        match ev {
            Event::SetParam { param, value, .. } => {
                assert_eq!(param, ParamId::FilterCutoff);
                assert!((value - 20.0).abs() < 1e-3, "value = {value}");
            }
            other => panic!("expected SetParam, got {other:?}"),
        }
    }

    #[test]
    fn cc_resonance_midpoint() {
        // CC 71 (resonance), value 64 -> ~0.504 within 0..1.
        let ev = parse_message(&[0xB0, 71, 64], 0).unwrap();
        match ev {
            Event::SetParam { param, value, .. } => {
                assert_eq!(param, ParamId::FilterResonance);
                assert!((value - 64.0 / 127.0).abs() < 1e-6, "value = {value}");
            }
            other => panic!("expected SetParam, got {other:?}"),
        }
    }

    #[test]
    fn cc_master_volume_special_case() {
        // CC 7 maps to MasterVolume, not a SetParam.
        let ev = parse_message(&[0xB0, 7, 127], 0).unwrap();
        match ev {
            Event::MasterVolume(v) => assert!((v - 1.0).abs() < 1e-6, "v = {v}"),
            other => panic!("expected MasterVolume, got {other:?}"),
        }
    }

    #[test]
    fn cc_all_notes_off() {
        for control in [120u8, 123u8] {
            let ev = parse_message(&[0xB1, control, 0], 0).unwrap();
            match ev {
                Event::AllNotesOff { channel } => assert_eq!(channel, 1),
                other => panic!("expected AllNotesOff, got {other:?}"),
            }
        }
    }

    #[test]
    fn unmapped_cc_is_ignored() {
        assert!(parse_message(&[0xB0, 3, 100], 0).is_none());
    }

    #[test]
    fn short_and_empty_messages_are_ignored() {
        assert!(parse_message(&[], 0).is_none());
        assert!(parse_message(&[0x90], 0).is_none());
        assert!(parse_message(&[0x90, 60], 0).is_none());
    }

    #[test]
    fn scale_cc_endpoints() {
        assert!((scale_cc(0, 0.001, 2.0) - 0.001).abs() < 1e-6);
        assert!((scale_cc(127, 0.001, 2.0) - 2.0).abs() < 1e-6);
    }
}
