//! egui/eframe GUI for Rusted Moog — a polished, dark "Moog Subsequent 37"
//! inspired virtual-analog control panel.
//!
//! Everything lives in this single file on purpose (custom rotary knob widget,
//! VU meter, virtual keyboard, preset/channel selectors and the dark theme).

use crate::shared::{EventSender, SharedState};
use egui::{pos2, vec2, Align2, Color32, Context, FontId, Pos2, Rect, RichText, Sense, Stroke, Ui};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use voog_dsp::event::ParamId;
use voog_dsp::params::{GlideMode, LfoDest, NoiseType, Patch, Waveform};
use voog_dsp::Event;

// ── Moog-inspired palette ──────────────────────────────────────────────────
const BG: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);
const PANEL_BG: Color32 = Color32::from_rgb(0x24, 0x24, 0x24);
const BORDER: Color32 = Color32::from_rgb(0x3a, 0x3a, 0x3a);
const ACCENT: Color32 = Color32::from_rgb(0xe8, 0xa0, 0x25); // amber
const ACCENT_DIM: Color32 = Color32::from_rgb(0xb3, 0x7a, 0x1a);
const CREAM: Color32 = Color32::from_rgb(0xd4, 0xc9, 0xa8);
const CREAM_DIM: Color32 = Color32::from_rgb(0x8a, 0x80, 0x68);
const TROUGH: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
const HEADER_BG: Color32 = Color32::from_rgb(0x2c, 0x1e, 0x10);
const WHITE_KEY: Color32 = Color32::from_rgb(0xe8, 0xe4, 0xd8);
const BLACK_KEY: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x1a);

// ── Enum <-> label tables ──────────────────────────────────────────────────
const WAVEFORMS: [(Waveform, &str); 4] = [
    (Waveform::Sine, "SINE"),
    (Waveform::Saw, "SAW"),
    (Waveform::Square, "SQUARE"),
    (Waveform::Triangle, "TRIANGLE"),
];
const NOISE_TYPES: [(NoiseType, &str); 2] =
    [(NoiseType::White, "WHITE"), (NoiseType::Pink, "PINK")];
const LFO_DESTS: [(LfoDest, &str); 3] = [
    (LfoDest::Filter, "FILTER"),
    (LfoDest::Pitch, "PITCH"),
    (LfoDest::Amp, "AMP"),
];
const GLIDE_MODES: [(GlideMode, &str); 3] = [
    (GlideMode::Off, "OFF"),
    (GlideMode::Always, "ALWAYS"),
    (GlideMode::Legato, "LEGATO"),
];

/// QWERTY -> semitone-offset mapping (matches the Python app).
const KEY_MAP: [(egui::Key, i32); 16] = [
    (egui::Key::A, 0),
    (egui::Key::W, 1),
    (egui::Key::S, 2),
    (egui::Key::E, 3),
    (egui::Key::D, 4),
    (egui::Key::F, 5),
    (egui::Key::T, 6),
    (egui::Key::G, 7),
    (egui::Key::Y, 8),
    (egui::Key::H, 9),
    (egui::Key::U, 10),
    (egui::Key::J, 11),
    (egui::Key::K, 12),
    (egui::Key::O, 13),
    (egui::Key::L, 14),
    (egui::Key::P, 15),
];

// ── Value formatting ───────────────────────────────────────────────────────
#[derive(Clone, Copy)]
enum KFmt {
    Int,
    One,
    Percent,
    Hz,
    Time,
    Rate,
}

fn fmt_value(v: f32, f: KFmt) -> String {
    match f {
        KFmt::Int => format!("{}", v.round() as i32),
        KFmt::One => format!("{v:.1}"),
        KFmt::Percent => format!("{}%", (v * 100.0).round() as i32),
        KFmt::Hz => {
            if v < 1000.0 {
                format!("{}Hz", v.round() as i32)
            } else {
                format!("{:.1}k", v / 1000.0)
            }
        }
        KFmt::Time => {
            if v < 0.1 {
                format!("{:.0}ms", v * 1000.0)
            } else {
                format!("{v:.2}s")
            }
        }
        KFmt::Rate => format!("{v:.1}Hz"),
    }
}

// ── Custom rotary knob widget ──────────────────────────────────────────────

const KNOB_ARC_START: f32 = 2.356_194_5; // 135° (7 o'clock), screen coords (y down)
const KNOB_ARC_SWEEP: f32 = 4.712_389; // 270° clockwise
const KNOB_DRAG_PX: f32 = 200.0;

fn ratio_to_value(r: f32, min: f32, max: f32, log: bool) -> f32 {
    if log && min > 0.0 {
        min * (max / min).powf(r)
    } else {
        min + r * (max - min)
    }
}

fn value_to_ratio(v: f32, min: f32, max: f32, log: bool) -> f32 {
    let r = if log && min > 0.0 {
        (v.max(min).ln() - min.ln()) / (max.ln() - min.ln())
    } else {
        (v - min) / (max - min)
    };
    r.clamp(0.0, 1.0)
}

/// Draw a rotary knob. Returns `true` when the value changed this frame.
#[allow(clippy::too_many_arguments)]
fn knob(
    ui: &mut Ui,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    log: bool,
    default: f32,
    fmt: KFmt,
) -> bool {
    let (resp, painter) = ui.allocate_painter(vec2(62.0, 74.0), Sense::click_and_drag());
    let rect = resp.rect;
    let cx = rect.center().x;
    let top = rect.top();
    let r_track = 22.0_f32;
    let center = pos2(cx, top + 14.0 + r_track);

    let mut changed = false;
    let mut ratio = value_to_ratio(*value, min, max, log);

    if resp.dragged() {
        let dy = resp.drag_delta().y;
        ratio = (ratio - dy / KNOB_DRAG_PX).clamp(0.0, 1.0);
        *value = ratio_to_value(ratio, min, max, log);
        changed = true;
    }
    if resp.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            ratio = (ratio + scroll.signum() * 0.03).clamp(0.0, 1.0);
            *value = ratio_to_value(ratio, min, max, log);
            changed = true;
        }
    }
    if resp.double_clicked() {
        *value = default;
        ratio = value_to_ratio(*value, min, max, log);
        changed = true;
    }

    // Label
    painter.text(
        pos2(cx, top + 6.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        CREAM_DIM,
    );

    // Track arc (dim, full sweep)
    let arc = |t0: f32, t1: f32| -> Vec<Pos2> {
        let n = ((t1 - t0).abs() * 28.0).ceil().max(2.0) as usize;
        (0..=n)
            .map(|k| {
                let t = t0 + (t1 - t0) * (k as f32 / n as f32);
                let a = KNOB_ARC_START + t * KNOB_ARC_SWEEP;
                pos2(center.x + r_track * a.cos(), center.y + r_track * a.sin())
            })
            .collect()
    };
    painter.add(egui::Shape::line(arc(0.0, 1.0), Stroke::new(3.0, TROUGH)));
    if ratio > 0.004 {
        painter.add(egui::Shape::line(arc(0.0, ratio), Stroke::new(3.0, ACCENT)));
    }

    // Knob body
    let r_body = 15.0;
    painter.circle_filled(center, r_body, Color32::from_rgb(0x2c, 0x2c, 0x2c));
    painter.circle_stroke(
        center,
        r_body,
        Stroke::new(1.5, Color32::from_rgb(0x44, 0x44, 0x44)),
    );
    painter.circle_filled(center, 2.0, Color32::from_rgb(0x40, 0x40, 0x40));

    // Pointer
    let a = KNOB_ARC_START + ratio * KNOB_ARC_SWEEP;
    let p_in = pos2(center.x + 5.0 * a.cos(), center.y + 5.0 * a.sin());
    let p_out = pos2(center.x + 13.5 * a.cos(), center.y + 13.5 * a.sin());
    painter.line_segment([p_in, p_out], Stroke::new(2.5, ACCENT));

    // Value text
    painter.text(
        pos2(cx, center.y + r_track + 9.0),
        Align2::CENTER_CENTER,
        fmt_value(*value, fmt),
        FontId::proportional(9.5),
        CREAM,
    );

    changed
}

/// A labelled combo box over an enum. Returns `true` when the choice changed.
fn combo<T: PartialEq + Copy>(ui: &mut Ui, id: &str, cur: &mut T, opts: &[(T, &str)]) -> bool {
    let sel = opts
        .iter()
        .find(|(v, _)| *v == *cur)
        .map(|(_, n)| *n)
        .unwrap_or("");
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(sel)
        .width(96.0)
        .show_ui(ui, |ui| {
            for (v, n) in opts {
                if ui.selectable_value(cur, *v, *n).changed() {
                    changed = true;
                }
            }
        });
    changed
}

/// A framed panel with an amber header title.
fn panel(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    egui::Frame::none()
        .fill(PANEL_BG)
        .rounding(6.0)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).color(ACCENT).size(11.0).strong());
            ui.add_space(5.0);
            add(ui);
        });
}

fn note_at(p: Pos2, blacks: &[(Rect, i32)], whites: &[(Rect, i32)]) -> Option<i32> {
    for (r, n) in blacks {
        if r.contains(p) {
            return Some(*n);
        }
    }
    for (r, n) in whites {
        if r.contains(p) {
            return Some(*n);
        }
    }
    None
}

// ── Application ────────────────────────────────────────────────────────────

struct App {
    tx: EventSender,
    shared: Arc<SharedState>,
    presets: Vec<Patch>,
    patch: Patch,
    channel: u8,
    preset_idx: Option<usize>,
    // f32 shadows for the integer osc params so knob dragging stays smooth.
    osc_oct: [f32; 3],
    osc_semi: [f32; 3],
    // virtual keyboard state
    kbd_octave: i32,
    active_notes: HashSet<i32>,
    pc_notes: HashMap<egui::Key, i32>,
    mouse_note: Option<i32>,
    // meters
    vu_db: f32,
    peak_db: f32,
}

impl App {
    fn new(
        cc: &eframe::CreationContext<'_>,
        tx: EventSender,
        shared: Arc<SharedState>,
        presets: Vec<Patch>,
    ) -> Self {
        Self::install_theme(&cc.egui_ctx);
        let patch = Patch::default();
        let mut app = Self {
            tx,
            shared,
            presets,
            patch,
            channel: 0,
            preset_idx: None,
            osc_oct: [0.0; 3],
            osc_semi: [0.0; 3],
            kbd_octave: 4,
            active_notes: HashSet::new(),
            pc_notes: HashMap::new(),
            mouse_note: None,
            vu_db: -60.0,
            peak_db: -60.0,
        };
        app.sync_shadows();
        app
    }

    fn install_theme(ctx: &Context) {
        let mut v = egui::Visuals::dark();
        v.override_text_color = Some(CREAM);
        v.panel_fill = BG;
        v.window_fill = BG;
        v.faint_bg_color = PANEL_BG;
        v.extreme_bg_color = TROUGH;
        v.widgets.noninteractive.bg_fill = PANEL_BG;
        v.widgets.inactive.bg_fill = Color32::from_rgb(0x33, 0x33, 0x33);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x33, 0x33, 0x33);
        v.widgets.hovered.bg_fill = Color32::from_rgb(0x45, 0x45, 0x45);
        v.widgets.active.bg_fill = ACCENT_DIM;
        v.selection.bg_fill = ACCENT_DIM;
        v.selection.stroke = Stroke::new(1.0, ACCENT);
        v.hyperlink_color = ACCENT;
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.rounding = egui::Rounding::same(4.0);
        }
        ctx.set_visuals(v);

        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = vec2(6.0, 6.0);
        style.spacing.button_padding = vec2(8.0, 4.0);
        ctx.set_style(style);
    }

    // ── engine helpers ──────────────────────────────────────────────────
    fn send(&self, ev: Event) {
        let _ = self.tx.send(ev);
    }

    fn set_param(&mut self, param: ParamId, value: f32) {
        self.patch.apply_param(param, value);
        self.send(Event::SetParam {
            channel: self.channel,
            param,
            value,
        });
    }

    fn note_on(&mut self, note: i32) {
        self.active_notes.insert(note);
        self.send(Event::NoteOn {
            channel: self.channel,
            note,
            velocity: 100,
        });
    }

    fn note_off(&mut self, note: i32) {
        self.active_notes.remove(&note);
        self.send(Event::NoteOff {
            channel: self.channel,
            note,
        });
    }

    fn set_channel(&mut self, ch: u8) {
        if ch == self.channel {
            return;
        }
        self.send(Event::AllNotesOff {
            channel: self.channel,
        });
        self.active_notes.clear();
        self.pc_notes.clear();
        self.mouse_note = None;
        self.channel = ch;
    }

    fn sync_shadows(&mut self) {
        for (k, o) in self.patch.oscillators.iter().enumerate().take(3) {
            self.osc_oct[k] = o.octave as f32;
            self.osc_semi[k] = o.semitone as f32;
        }
    }

    fn load_preset(&mut self, i: usize) {
        self.patch = self.presets[i].clone();
        self.preset_idx = Some(i);
        self.sync_shadows();
        self.send(Event::LoadPatch {
            channel: self.channel,
            patch: Box::new(self.patch.clone()),
        });
        self.send(Event::MasterVolume(self.patch.master_volume));
    }

    // ── input ───────────────────────────────────────────────────────────
    fn process_keyboard(&mut self, ctx: &Context) {
        let keys_down = ctx.input(|i| i.keys_down.clone());
        for (k, semi) in KEY_MAP {
            let is_down = keys_down.contains(&k);
            let was_down = self.pc_notes.contains_key(&k);
            if is_down && !was_down {
                let note = (self.kbd_octave + 1) * 12 + semi;
                self.pc_notes.insert(k, note);
                self.note_on(note);
            } else if !is_down && was_down {
                if let Some(note) = self.pc_notes.remove(&k) {
                    self.note_off(note);
                }
            }
        }
    }

    fn update_meters(&mut self) {
        let peak = self.shared.peak();
        self.vu_db = if peak < 1e-6 {
            -60.0
        } else {
            (20.0 * peak.log10()).clamp(-60.0, 6.0)
        };
        if self.vu_db > self.peak_db {
            self.peak_db = self.vu_db;
        } else {
            self.peak_db = (self.peak_db - 0.3).max(-60.0);
        }
    }

    // ── layout ──────────────────────────────────────────────────────────
    fn header(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new("RUSTED MOOG")
                    .color(ACCENT)
                    .size(24.0)
                    .strong(),
            );
            ui.add_space(10.0);
            ui.label(
                RichText::new("VIRTUAL ANALOG SYNTHESIZER")
                    .color(CREAM_DIM)
                    .size(10.0),
            );
        });
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("PATCH").color(CREAM_DIM).size(10.0));

            let cur_name = self
                .preset_idx
                .and_then(|i| self.presets.get(i))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| self.patch.name.clone());
            let mut chosen: Option<usize> = None;
            egui::ComboBox::from_id_salt("preset")
                .selected_text(cur_name)
                .width(180.0)
                .show_ui(ui, |ui| {
                    for (i, p) in self.presets.iter().enumerate() {
                        if ui
                            .selectable_label(self.preset_idx == Some(i), &p.name)
                            .clicked()
                        {
                            chosen = Some(i);
                        }
                    }
                });
            if let Some(i) = chosen {
                self.load_preset(i);
            }

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(RichText::new("CHANNEL").color(CREAM_DIM).size(10.0));
            for ch in 0..4u8 {
                let active = ch == self.channel;
                let btn = egui::Button::new(
                    RichText::new(format!("{}", ch + 1))
                        .color(if active { Color32::BLACK } else { CREAM })
                        .strong(),
                )
                .fill(if active {
                    ACCENT
                } else {
                    Color32::from_rgb(0x3a, 0x3a, 0x3a)
                })
                .min_size(vec2(30.0, 24.0));
                if ui.add(btn).clicked() {
                    self.set_channel(ch);
                }
            }
        });
    }

    fn body(&mut self, ui: &mut Ui) {
        // Row 1: three oscillators + noise.
        ui.columns(4, |c| {
            self.osc_panel(&mut c[0], 0);
            self.osc_panel(&mut c[1], 1);
            self.osc_panel(&mut c[2], 2);
            self.noise_panel(&mut c[3]);
        });
        ui.add_space(6.0);
        // Row 2: filter + envelopes + LFO.
        ui.columns(4, |c| {
            self.filter_panel(&mut c[0]);
            self.adsr_panel(&mut c[1], "FILTER ENVELOPE", false);
            self.adsr_panel(&mut c[2], "AMP ENVELOPE", true);
            self.lfo_panel(&mut c[3]);
        });
        ui.add_space(6.0);
        // Row 3: glide + output/status.
        ui.columns(2, |c| {
            self.glide_panel(&mut c[0]);
            self.status_panel(&mut c[1]);
        });
    }

    fn osc_panel(&mut self, ui: &mut Ui, i: usize) {
        panel(ui, &format!("OSCILLATOR {}", i + 1), |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("WAVE").color(CREAM_DIM).size(9.0));
                let mut wf = self.patch.oscillators[i].waveform;
                if combo(ui, &format!("osc_wf_{i}"), &mut wf, &WAVEFORMS) {
                    self.patch.oscillators[i].waveform = wf;
                    self.send(Event::SetOscWaveform {
                        channel: self.channel,
                        osc: i,
                        waveform: wf,
                    });
                }
            });
            ui.horizontal(|ui| {
                let mut oct = self.osc_oct[i];
                if knob(ui, "OCT", &mut oct, -2.0, 2.0, false, 0.0, KFmt::Int) {
                    self.osc_oct[i] = oct;
                    self.set_param(ParamId::OscOctave(i), oct);
                }
                let mut semi = self.osc_semi[i];
                if knob(ui, "SEMI", &mut semi, -12.0, 12.0, false, 0.0, KFmt::Int) {
                    self.osc_semi[i] = semi;
                    self.set_param(ParamId::OscSemitone(i), semi);
                }
                let mut det = self.patch.oscillators[i].detune;
                if knob(ui, "DETUNE", &mut det, -50.0, 50.0, false, 0.0, KFmt::One) {
                    self.set_param(ParamId::OscDetune(i), det);
                }
                let mut lvl = self.patch.oscillators[i].level;
                if knob(ui, "LEVEL", &mut lvl, 0.0, 1.0, false, 1.0, KFmt::Percent) {
                    self.set_param(ParamId::OscLevel(i), lvl);
                }
            });
        });
    }

    fn noise_panel(&mut self, ui: &mut Ui) {
        panel(ui, "NOISE", |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("TYPE").color(CREAM_DIM).size(9.0));
                let mut nt = self.patch.noise.noise_type;
                if combo(ui, "noise_type", &mut nt, &NOISE_TYPES) {
                    self.patch.noise.noise_type = nt;
                    self.send(Event::SetNoiseType {
                        channel: self.channel,
                        noise_type: nt,
                    });
                }
            });
            ui.horizontal(|ui| {
                let mut lvl = self.patch.noise.level;
                if knob(ui, "LEVEL", &mut lvl, 0.0, 1.0, false, 0.0, KFmt::Percent) {
                    self.set_param(ParamId::NoiseLevel, lvl);
                }
            });
        });
    }

    fn filter_panel(&mut self, ui: &mut Ui) {
        panel(ui, "FILTER", |ui| {
            ui.horizontal(|ui| {
                let f = self.patch.filter;
                let mut cut = f.cutoff;
                if knob(
                    ui,
                    "CUTOFF",
                    &mut cut,
                    20.0,
                    20000.0,
                    true,
                    8000.0,
                    KFmt::Hz,
                ) {
                    self.set_param(ParamId::FilterCutoff, cut);
                }
                let mut res = f.resonance;
                if knob(ui, "RESO", &mut res, 0.0, 1.0, false, 0.0, KFmt::Percent) {
                    self.set_param(ParamId::FilterResonance, res);
                }
                let mut env = f.env_amount;
                if knob(ui, "ENV AMT", &mut env, 0.0, 48.0, false, 0.0, KFmt::One) {
                    self.set_param(ParamId::FilterEnvAmount, env);
                }
                let mut kt = f.key_tracking;
                if knob(ui, "KEY TRK", &mut kt, 0.0, 1.0, false, 0.0, KFmt::Percent) {
                    self.set_param(ParamId::FilterKeyTracking, kt);
                }
            });
        });
    }

    fn adsr_panel(&mut self, ui: &mut Ui, title: &str, amp: bool) {
        panel(ui, title, |ui| {
            ui.horizontal(|ui| {
                let a = if amp {
                    self.patch.amp_adsr
                } else {
                    self.patch.filter_adsr
                };
                let (pa, pd, ps, pr) = if amp {
                    (
                        ParamId::AmpAttack,
                        ParamId::AmpDecay,
                        ParamId::AmpSustain,
                        ParamId::AmpRelease,
                    )
                } else {
                    (
                        ParamId::FilterAttack,
                        ParamId::FilterDecay,
                        ParamId::FilterSustain,
                        ParamId::FilterRelease,
                    )
                };
                let mut at = a.attack;
                if knob(ui, "A", &mut at, 0.001, 2.0, true, 0.01, KFmt::Time) {
                    self.set_param(pa, at);
                }
                let mut dc = a.decay;
                if knob(ui, "D", &mut dc, 0.001, 2.0, true, 0.1, KFmt::Time) {
                    self.set_param(pd, dc);
                }
                let mut su = a.sustain;
                if knob(ui, "S", &mut su, 0.0, 1.0, false, 0.7, KFmt::Percent) {
                    self.set_param(ps, su);
                }
                let mut re = a.release;
                if knob(ui, "R", &mut re, 0.001, 3.0, true, 0.3, KFmt::Time) {
                    self.set_param(pr, re);
                }
            });
        });
    }

    fn lfo_panel(&mut self, ui: &mut Ui) {
        panel(ui, "LFO", |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("WAVE").color(CREAM_DIM).size(9.0));
                let mut wf = self.patch.lfo.waveform;
                if combo(ui, "lfo_wf", &mut wf, &WAVEFORMS) {
                    self.patch.lfo.waveform = wf;
                    self.send(Event::SetLfoWaveform {
                        channel: self.channel,
                        waveform: wf,
                    });
                }
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new("DEST").color(CREAM_DIM).size(9.0));
                let mut d = self.patch.lfo.destination;
                if combo(ui, "lfo_dest", &mut d, &LFO_DESTS) {
                    self.patch.lfo.destination = d;
                    self.send(Event::SetLfoDest {
                        channel: self.channel,
                        dest: d,
                    });
                }
            });
            ui.horizontal(|ui| {
                let mut rate = self.patch.lfo.rate;
                if knob(ui, "RATE", &mut rate, 0.1, 20.0, true, 1.0, KFmt::Rate) {
                    self.set_param(ParamId::LfoRate, rate);
                }
                let mut depth = self.patch.lfo.depth;
                if knob(ui, "DEPTH", &mut depth, 0.0, 1.0, false, 0.0, KFmt::Percent) {
                    self.set_param(ParamId::LfoDepth, depth);
                }
            });
        });
    }

    fn glide_panel(&mut self, ui: &mut Ui) {
        panel(ui, "GLIDE", |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("MODE").color(CREAM_DIM).size(9.0));
                let mut m = self.patch.glide.mode;
                if combo(ui, "glide_mode", &mut m, &GLIDE_MODES) {
                    self.patch.glide.mode = m;
                    self.send(Event::SetGlideMode {
                        channel: self.channel,
                        mode: m,
                    });
                }
                let mut t = self.patch.glide.time;
                if knob(ui, "TIME", &mut t, 0.0, 1.0, false, 0.0, KFmt::Time) {
                    self.set_param(ParamId::GlideTime, t);
                }
            });
        });
    }

    fn status_panel(&mut self, ui: &mut Ui) {
        panel(ui, "OUTPUT", |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("VOICES").color(CREAM_DIM).size(9.0));
                    ui.label(
                        RichText::new(format!("{}", self.shared.voices()))
                            .color(ACCENT)
                            .size(22.0)
                            .strong(),
                    );
                });
                ui.add_space(10.0);
                self.draw_vu(ui);
                ui.add_space(10.0);
                let mut m = self.patch.master_volume;
                if knob(ui, "MASTER", &mut m, 0.0, 1.0, false, 0.7, KFmt::Percent) {
                    self.patch.master_volume = m;
                    self.send(Event::MasterVolume(m));
                }
            });
        });
    }

    fn draw_vu(&mut self, ui: &mut Ui) {
        let (resp, painter) = ui.allocate_painter(vec2(26.0, 92.0), Sense::hover());
        let rect = resp.rect;
        painter.rect_filled(rect, 3.0, Color32::from_rgb(0x12, 0x12, 0x12));
        let h = rect.height();
        let bx1 = rect.left() + 4.0;
        let bx2 = rect.right() - 4.0;
        let level_ratio = ((self.vu_db + 60.0) / 60.0).clamp(0.0, 1.0);
        let top_fill = rect.bottom() - level_ratio * h;

        let seg = 4.0;
        let mut y = rect.bottom();
        while y > top_fill {
            let r = (rect.bottom() - y) / h;
            let col = if r < 0.6 {
                Color32::from_rgb(0x22, 0xcc, 0x44)
            } else if r < 0.8 {
                Color32::from_rgb(0xcc, 0xcc, 0x22)
            } else {
                Color32::from_rgb(0xcc, 0x33, 0x22)
            };
            let yt = (y - seg).max(top_fill);
            painter.rect_filled(
                Rect::from_min_max(pos2(bx1, yt), pos2(bx2, y - 1.0)),
                0.0,
                col,
            );
            y -= seg;
        }

        if self.peak_db > -59.0 {
            let pr = ((self.peak_db + 60.0) / 60.0).clamp(0.0, 1.0);
            let py = rect.bottom() - pr * h;
            painter.line_segment([pos2(bx1, py), pos2(bx2, py)], Stroke::new(2.0, ACCENT));
        }
    }

    fn keyboard(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("OCT -").clicked() {
                self.kbd_octave = (self.kbd_octave - 1).max(0);
            }
            ui.label(
                RichText::new(format!("OCTAVE {}", self.kbd_octave))
                    .color(ACCENT)
                    .strong(),
            );
            if ui.button("OCT +").clicked() {
                self.kbd_octave = (self.kbd_octave + 1).min(7);
            }
            ui.add_space(10.0);
            ui.label(
                RichText::new("keys:  A W S E D F T G Y H U J K O L P")
                    .color(CREAM_DIM)
                    .size(9.0),
            );
        });
        ui.add_space(4.0);

        let avail_w = ui.available_width();
        let (resp, painter) = ui.allocate_painter(vec2(avail_w, 120.0), Sense::click_and_drag());
        let rect = resp.rect;

        let num_oct = 3;
        let whites = [0, 2, 4, 5, 7, 9, 11];
        let total_white = num_oct * 7 + 1;
        let wk = rect.width() / total_white as f32;
        let bk_w = wk * 0.6;
        let bk_h = rect.height() * 0.62;
        let base = (self.kbd_octave + 1) * 12;

        // White key rects
        let mut white_notes: Vec<(Rect, i32)> = Vec::new();
        let mut wi = 0;
        for o in 0..num_oct {
            for &semi in &whites {
                let note = base + o * 12 + semi;
                let x1 = rect.left() + wi as f32 * wk;
                white_notes.push((
                    Rect::from_min_max(pos2(x1, rect.top()), pos2(x1 + wk, rect.bottom())),
                    note,
                ));
                wi += 1;
            }
        }
        let x1 = rect.left() + wi as f32 * wk;
        white_notes.push((
            Rect::from_min_max(pos2(x1, rect.top()), pos2(x1 + wk, rect.bottom())),
            base + num_oct * 12,
        ));

        // Black key rects
        let blacks = [(0usize, 1i32), (1, 3), (3, 6), (4, 8), (5, 10)];
        let mut black_notes: Vec<(Rect, i32)> = Vec::new();
        for o in 0..num_oct {
            for &(wn, semi) in &blacks {
                let note = base + o * 12 + semi;
                let cx = rect.left() + ((o as usize * 7 + wn + 1) as f32) * wk;
                black_notes.push((
                    Rect::from_min_max(
                        pos2(cx - bk_w / 2.0, rect.top()),
                        pos2(cx + bk_w / 2.0, rect.top() + bk_h),
                    ),
                    note,
                ));
            }
        }

        // Draw white keys
        let names = ["C", "", "D", "", "E", "F", "", "G", "", "A", "", "B"];
        for (r, note) in &white_notes {
            let on = self.active_notes.contains(note);
            let col = if on { ACCENT } else { WHITE_KEY };
            painter.rect(
                *r,
                2.0,
                col,
                Stroke::new(1.0, Color32::from_rgb(0xa0, 0x98, 0x80)),
            );
            let semi = (note.rem_euclid(12)) as usize;
            if semi == 0 {
                painter.text(
                    pos2(r.center().x, r.bottom() - 10.0),
                    Align2::CENTER_CENTER,
                    format!("C{}", note / 12 - 1),
                    FontId::proportional(8.0),
                    CREAM_DIM,
                );
            } else {
                painter.text(
                    pos2(r.center().x, r.bottom() - 10.0),
                    Align2::CENTER_CENTER,
                    names[semi],
                    FontId::proportional(8.0),
                    CREAM_DIM,
                );
            }
        }
        // Draw black keys on top
        for (r, note) in &black_notes {
            let on = self.active_notes.contains(note);
            let col = if on { ACCENT_DIM } else { BLACK_KEY };
            painter.rect(*r, 2.0, col, Stroke::new(1.0, Color32::BLACK));
        }

        // Mouse interaction
        if resp.is_pointer_button_down_on() {
            if let Some(p) = resp.interact_pointer_pos() {
                let note = note_at(p, &black_notes, &white_notes);
                if note != self.mouse_note {
                    if let Some(old) = self.mouse_note {
                        self.note_off(old);
                    }
                    if let Some(n) = note {
                        self.note_on(n);
                    }
                    self.mouse_note = note;
                }
            }
        } else if let Some(old) = self.mouse_note.take() {
            self.note_off(old);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.process_keyboard(ctx);
        self.update_meters();

        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none().fill(HEADER_BG).inner_margin(10.0))
            .show(ctx, |ui| self.header(ui));

        egui::TopBottomPanel::top("topbar")
            .frame(egui::Frame::none().fill(BG).inner_margin(8.0))
            .show(ctx, |ui| self.top_bar(ui));

        egui::TopBottomPanel::bottom("keyboard")
            .frame(egui::Frame::none().fill(PANEL_BG).inner_margin(8.0))
            .show(ctx, |ui| self.keyboard(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG).inner_margin(8.0))
            .show(ctx, |ui| self.body(ui));

        ctx.request_repaint();
    }
}

/// Run the GUI on the main thread (blocks until the window closes).
pub fn run(tx: EventSender, shared: Arc<SharedState>, presets: Vec<Patch>) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rusted Moog")
            .with_inner_size([1060.0, 780.0])
            .with_min_inner_size([840.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rusted Moog",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc, tx, shared, presets)))),
    )
}
