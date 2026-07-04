//! egui/eframe GUI for Rusted Moog — restyled to look like a real
//! **Moog Minimoog Model D** hardware panel.
//!
//! Everything lives in this single file on purpose: the brushed-aluminum rotary
//! knob widget, red rocker switches, segmented waveform selectors, pitch/mod
//! wheels, the VU meter, the classic keyboard, preset/channel selectors and the
//! vintage-hardware theme. Only the *look* changed — every control still emits
//! exactly the same `Event`s as before.

use crate::shared::{EventSender, SharedState};
use egui::{pos2, vec2, Align2, Color32, Context, FontId, Pos2, Rect, RichText, Sense, Stroke, Ui};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use voog_dsp::event::ParamId;
use voog_dsp::params::{GlideMode, LfoDest, NoiseType, Patch, Waveform};
use voog_dsp::Event;

// ── Minimoog-inspired palette ──────────────────────────────────────────────
// Dark backdrop behind the whole instrument.
const BG: Color32 = Color32::from_rgb(0x10, 0x11, 0x12);

// Oak wood side cheeks (base / highlight / shadow for vertical grain).
const WOOD: Color32 = Color32::from_rgb(0x7a, 0x52, 0x30);
const WOOD_HI: Color32 = Color32::from_rgb(0x9a, 0x6a, 0x40);
const WOOD_LO: Color32 = Color32::from_rgb(0x5c, 0x3d, 0x22);

// Near-black anodized-aluminium panel (matte, cool charcoal, top→bottom).
const PANEL_TOP: Color32 = Color32::from_rgb(0x22, 0x26, 0x2a);
const PANEL_BOT: Color32 = Color32::from_rgb(0x17, 0x1a, 0x1e);
const PANEL_BG: Color32 = Color32::from_rgb(0x1e, 0x22, 0x26);

// Engraved dividers: a dark hairline beside a lighter hairline reads as an
// incised groove in the metal.
const DIV_DARK: Color32 = Color32::from_rgb(0x0c, 0x0e, 0x10);
const DIV_LIGHT: Color32 = Color32::from_rgb(0x3a, 0x40, 0x46);

// Text — white silkscreen.
const HEADER_TEXT: Color32 = Color32::from_rgb(0xe8, 0xe8, 0xe6);
const CREAM: Color32 = Color32::from_rgb(0xe0, 0xe0, 0xdc);
const CREAM_DIM: Color32 = Color32::from_rgb(0x9a, 0x9c, 0x9a);

const TROUGH: Color32 = Color32::from_rgb(0x14, 0x16, 0x18);

// Amber "lit" accent — reserved for active / selected states.
const ACCENT: Color32 = Color32::from_rgb(0xe8, 0xa0, 0x25);
const ACCENT_DIM: Color32 = Color32::from_rgb(0xb3, 0x7a, 0x1a);

// Matte maroon-red paddle / bat toggle lever.
const PADDLE_BODY: Color32 = Color32::from_rgb(0x7a, 0x26, 0x22);
const PADDLE_HI: Color32 = Color32::from_rgb(0xa8, 0x3b, 0x34);
const PADDLE_DARK: Color32 = Color32::from_rgb(0x3a, 0x12, 0x10);

// Keyboard.
const WHITE_KEY: Color32 = Color32::from_rgb(0xec, 0xe7, 0xd8);
const BLACK_KEY: Color32 = Color32::from_rgb(0x14, 0x14, 0x14);

// ── Enum <-> label tables ──────────────────────────────────────────────────
const WAVEFORMS: [(Waveform, &str); 4] = [
    (Waveform::Sine, "SINE"),
    (Waveform::Saw, "SAW"),
    (Waveform::Square, "SQR"),
    (Waveform::Triangle, "TRI"),
];
const NOISE_TYPES: [(NoiseType, &str); 2] =
    [(NoiseType::White, "WHITE"), (NoiseType::Pink, "PINK")];
const LFO_DESTS: [(LfoDest, &str); 3] = [
    (LfoDest::Filter, "FILT"),
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

// ── Small painting helpers ─────────────────────────────────────────────────

/// Fill `rect` with a 4-corner interpolated gradient.
fn gradient_rect(
    painter: &egui::Painter,
    rect: Rect,
    tl: Color32,
    tr: Color32,
    br: Color32,
    bl: Color32,
) {
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), tl);
    mesh.colored_vertex(rect.right_top(), tr);
    mesh.colored_vertex(rect.right_bottom(), br);
    mesh.colored_vertex(rect.left_bottom(), bl);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    painter.add(mesh);
}

/// Vertical (top→bottom) gradient fill.
fn vgrad(painter: &egui::Painter, rect: Rect, top: Color32, bot: Color32) {
    gradient_rect(painter, rect, top, top, bot, bot);
}

/// Horizontal (left→right) gradient fill.
fn hgrad(painter: &egui::Painter, rect: Rect, left: Color32, right: Color32) {
    gradient_rect(painter, rect, left, right, right, left);
}

/// Letter-space a header string ("OSC" -> "O S C").
fn spaced(s: &str) -> String {
    s.chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Paint an oak-wood side cheek: base fill, a soft rounded sheen and a handful
/// of darker vertical grain streaks.
fn paint_wood(painter: &egui::Painter, rect: Rect, tex: Option<&egui::TextureHandle>) {
    if let Some(tex) = tex {
        // Real oak photo: sample a vertical slice so the grain reads as vertical
        // on the tall, narrow cheek instead of being smeared horizontally.
        let uv = Rect::from_min_max(pos2(0.36, 0.02), pos2(0.60, 0.98));
        painter.image(tex.id(), rect, uv, Color32::WHITE);
        // Slight recess darkening for the cylindrical, framed feel.
        painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(18, 10, 3, 46));
    } else {
        painter.rect_filled(rect, 0.0, WOOD);
        // Rounded cylindrical sheen across the width (dark edges, lit centre).
        let mid = rect.center().x;
        let lh = Rect::from_min_max(rect.left_top(), pos2(mid, rect.bottom()));
        let rh = Rect::from_min_max(pos2(mid, rect.top()), rect.right_bottom());
        hgrad(painter, lh, WOOD_LO, WOOD_HI);
        hgrad(painter, rh, WOOD_HI, WOOD_LO);
        // Vertical grain streaks.
        let w = rect.width();
        let streaks = [0.18f32, 0.34, 0.5, 0.63, 0.78, 0.9];
        for (k, t) in streaks.iter().enumerate() {
            let x = rect.left() + w * t;
            let col = if k % 2 == 0 { WOOD_LO } else { WOOD_HI };
            painter.line_segment(
                [
                    pos2(x, rect.top() + 2.0),
                    pos2(x + 0.6, rect.bottom() - 2.0),
                ],
                Stroke::new(1.0, col),
            );
        }
    }
    // Inner shadow at both edges.
    painter.line_segment(
        [rect.left_top(), rect.left_bottom()],
        Stroke::new(2.0, Color32::from_rgba_unmultiplied(0, 0, 0, 90)),
    );
    painter.line_segment(
        [rect.right_top(), rect.right_bottom()],
        Stroke::new(2.0, Color32::from_rgba_unmultiplied(0, 0, 0, 90)),
    );
}

// ── Rotary knob widget (the centerpiece) ───────────────────────────────────

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

/// Draw a Minimoog-style rotary knob: a fluted black skirt, a prominent bright
/// brushed-aluminium domed cap, a crisp white pointer, and a faint silkscreened
/// arc scale (ticks + 0/10 numbering) printed on the panel around it. Returns
/// `true` when the value changed this frame.
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
    let (resp, painter) = ui.allocate_painter(vec2(56.0, 86.0), Sense::click_and_drag());
    let rect = resp.rect;
    let cx = rect.center().x;
    let top = rect.top();
    let r_skirt = 19.0_f32;
    let center = pos2(cx, top + 15.0 + r_skirt);

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

    // Label above.
    painter.text(
        pos2(cx, top + 6.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        HEADER_TEXT,
    );

    // (0) Silkscreened arc scale printed on the panel (outside the skirt):
    // faint tick marks around the 270° sweep with 0 / 10 numbering.
    let r_scale = 23.0;
    let scale_ticks = 11;
    for k in 0..scale_ticks {
        let t = k as f32 / (scale_ticks - 1) as f32;
        let a = KNOB_ARC_START + t * KNOB_ARC_SWEEP;
        let (c, s) = (a.cos(), a.sin());
        let long = k == 0 || k == scale_ticks - 1 || k == (scale_ticks - 1) / 2;
        let r0 = if long { r_scale - 2.5 } else { r_scale - 1.5 };
        painter.line_segment(
            [
                pos2(center.x + r0 * c, center.y + r0 * s),
                pos2(
                    center.x + (r_scale + 0.5) * c,
                    center.y + (r_scale + 0.5) * s,
                ),
            ],
            Stroke::new(1.0, Color32::from_rgb(0x6a, 0x6c, 0x6a)),
        );
    }
    let num_r = r_scale + 4.0;
    for (k, txt) in [(0usize, "0"), (scale_ticks - 1, "10")] {
        let t = k as f32 / (scale_ticks - 1) as f32;
        let a = KNOB_ARC_START + t * KNOB_ARC_SWEEP;
        painter.text(
            pos2(center.x + num_r * a.cos(), center.y + num_r * a.sin()),
            Align2::CENTER_CENTER,
            txt,
            FontId::proportional(7.0),
            Color32::from_rgb(0x7a, 0x7c, 0x7a),
        );
    }

    // (1) Fluted / scalloped black skirt: radial ridges alternating between a
    // faint highlight and shadow so it reads as a gripped edge.
    painter.circle_filled(center, r_skirt, Color32::from_rgb(0x0a, 0x0a, 0x0b));
    let flutes = 22;
    for k in 0..flutes {
        let a = k as f32 / flutes as f32 * std::f32::consts::TAU;
        let (c, s) = (a.cos(), a.sin());
        let col = if k % 2 == 0 {
            Color32::from_rgb(0x38, 0x38, 0x3a)
        } else {
            Color32::from_rgb(0x08, 0x08, 0x09)
        };
        painter.line_segment(
            [
                pos2(center.x + 13.5 * c, center.y + 13.5 * s),
                pos2(
                    center.x + (r_skirt - 0.5) * c,
                    center.y + (r_skirt - 0.5) * s,
                ),
            ],
            Stroke::new(1.6, col),
        );
    }
    painter.circle_stroke(center, r_skirt, Stroke::new(1.0, Color32::BLACK));

    // (2) Black body ring beneath the cap.
    painter.circle_filled(center, 13.5, Color32::from_rgb(0x16, 0x16, 0x18));

    // (3) Prominent bright brushed-aluminium domed cap: concentric fills from a
    // bright centre out to a darker rim, plus faint radial brush streaks and a
    // hot highlight so the dome reads as the centrepiece.
    let r_cap = 12.5_f32;
    let caps: [(f32, Color32); 5] = [
        (r_cap, Color32::from_rgb(0x84, 0x86, 0x88)),
        (10.5, Color32::from_rgb(0xa8, 0xaa, 0xac)),
        (8.0, Color32::from_rgb(0xc8, 0xca, 0xcc)),
        (5.0, Color32::from_rgb(0xe4, 0xe6, 0xe8)),
        (2.2, Color32::from_rgb(0xf4, 0xf5, 0xf6)),
    ];
    for (r, c) in caps {
        painter.circle_filled(center, r, c);
    }
    // Radial brushed streaks.
    for k in 0..28 {
        let a = k as f32 / 28.0 * std::f32::consts::TAU;
        let (c, s) = (a.cos(), a.sin());
        let shade = if k % 2 == 0 {
            Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 26)
        } else {
            Color32::from_rgba_unmultiplied(0x00, 0x00, 0x00, 20)
        };
        painter.line_segment(
            [
                pos2(center.x + 2.0 * c, center.y + 2.0 * s),
                pos2(center.x + (r_cap - 0.6) * c, center.y + (r_cap - 0.6) * s),
            ],
            Stroke::new(0.8, shade),
        );
    }
    painter.circle_filled(
        pos2(center.x - 3.2, center.y - 3.8),
        3.0,
        Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 120),
    );
    painter.circle_stroke(
        center,
        r_cap,
        Stroke::new(1.0, Color32::from_rgb(0x2a, 0x2a, 0x2c)),
    );

    // (4) White pointer from cap centre to the skirt edge, rotating with value.
    let a = KNOB_ARC_START + ratio * KNOB_ARC_SWEEP;
    let (c, s) = (a.cos(), a.sin());
    let p_in = pos2(center.x + 3.0 * c, center.y + 3.0 * s);
    let p_out = pos2(
        center.x + (r_skirt - 1.5) * c,
        center.y + (r_skirt - 1.5) * s,
    );
    painter.line_segment(
        [p_in, p_out],
        Stroke::new(2.2, Color32::from_rgb(0xf6, 0xf6, 0xf2)),
    );

    // (5) Value below.
    painter.text(
        pos2(cx, center.y + r_skirt + 9.0),
        Align2::CENTER_CENTER,
        fmt_value(*value, fmt),
        FontId::proportional(9.5),
        CREAM,
    );

    changed
}

// ── Red paddle / bat toggle switch ─────────────────────────────────────────

/// A matte-red 3D paddle (bat) toggle lever. The lever tilts to one side for
/// OFF and the other for ON, casts a soft drop shadow, and has tiny white
/// "OFF"/"ON" silkscreen labels beside it. Returns `true` when toggled.
fn rocker(ui: &mut Ui, label: &str, on: &mut bool) -> bool {
    let (resp, painter) = ui.allocate_painter(vec2(56.0, 44.0), Sense::click());
    let rect = resp.rect;

    let mut changed = false;
    if resp.clicked() {
        *on = !*on;
        changed = true;
    }

    // Recessed mounting plate.
    let plate = Rect::from_center_size(pos2(rect.center().x, rect.top() + 17.0), vec2(30.0, 28.0));
    painter.rect_filled(plate, 3.0, Color32::from_rgb(0x0a, 0x0b, 0x0c));
    painter.rect_stroke(plate, 3.0, Stroke::new(1.0, Color32::BLACK));

    // Tiny OFF / ON silkscreen labels, the active side lit white.
    painter.text(
        pos2(plate.left() - 2.0, plate.top() + 4.0),
        Align2::RIGHT_CENTER,
        "OFF",
        FontId::proportional(7.0),
        if *on { CREAM_DIM } else { HEADER_TEXT },
    );
    painter.text(
        pos2(plate.right() + 2.0, plate.top() + 4.0),
        Align2::LEFT_CENTER,
        "ON",
        FontId::proportional(7.0),
        if *on { HEADER_TEXT } else { CREAM_DIM },
    );

    // Lever geometry: a rounded bat pivoting near the plate bottom, tilting
    // ~20° toward ON (right) or OFF (left).
    let pivot = pos2(plate.center().x, plate.bottom() - 5.0);
    let ang: f32 = if *on { 0.36 } else { -0.36 };
    let (s, c) = ang.sin_cos();
    let up = vec2(s, -c); // (0,-1) rotated by ang
    let perp = vec2(c, s);
    let len = 19.0;
    let half_w = 4.5;
    let tip = pivot + up * len;
    let base_l = pivot + perp * half_w;
    let base_r = pivot - perp * half_w;
    let tip_l = tip + perp * half_w;
    let tip_r = tip - perp * half_w;

    // Soft drop shadow.
    let off = vec2(2.2, 3.0);
    painter.add(egui::Shape::convex_polygon(
        vec![base_l + off, tip_l + off, tip_r + off, base_r + off],
        Color32::from_rgba_unmultiplied(0, 0, 0, 90),
        Stroke::NONE,
    ));
    painter.circle_filled(
        tip + off,
        half_w,
        Color32::from_rgba_unmultiplied(0, 0, 0, 90),
    );

    // Lever body (matte maroon) + rounded tip.
    painter.add(egui::Shape::convex_polygon(
        vec![base_l, tip_l, tip_r, base_r],
        PADDLE_BODY,
        Stroke::new(1.0, PADDLE_DARK),
    ));
    painter.circle_filled(tip, half_w, PADDLE_BODY);
    painter.circle_stroke(tip, half_w, Stroke::new(1.0, PADDLE_DARK));

    // Highlight edge along the lit face + a small specular dot on the tip.
    painter.line_segment([base_l, tip_l], Stroke::new(1.5, PADDLE_HI));
    painter.circle_filled(tip - up * 1.2 - perp * 1.4, 1.5, PADDLE_HI);

    // Switch name below the plate.
    painter.text(
        pos2(rect.center().x, rect.bottom() - 5.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(8.0),
        CREAM_DIM,
    );

    changed
}

// ── Segmented metal switch strip (enum selector) ───────────────────────────

/// A compact metal segmented switch: one small labelled cell per option, the
/// active cell lit amber. Returns `true` when the choice changed.
fn segmented<T: PartialEq + Copy>(ui: &mut Ui, cur: &mut T, opts: &[(T, &str)]) -> bool {
    let w = ui.available_width().min(210.0);
    let h = 20.0;
    let (resp, painter) = ui.allocate_painter(vec2(w, h), Sense::click());
    let rect = resp.rect;
    let n = opts.len().max(1);
    let cw = rect.width() / n as f32;
    let click = if resp.clicked() {
        resp.interact_pointer_pos()
    } else {
        None
    };

    let mut changed = false;
    for (i, (v, name)) in opts.iter().enumerate() {
        let x0 = rect.left() + i as f32 * cw;
        let cell = Rect::from_min_size(pos2(x0, rect.top()), vec2(cw, h));
        let active = *v == *cur;
        let inner = cell.shrink(1.0);
        if active {
            vgrad(
                &painter,
                inner,
                Color32::from_rgb(0x5c, 0x50, 0x2c),
                Color32::from_rgb(0x3a, 0x31, 0x1a),
            );
            painter.line_segment(
                [
                    pos2(inner.left() + 2.0, inner.top() + 1.5),
                    pos2(inner.right() - 2.0, inner.top() + 1.5),
                ],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(0xff, 0xd0, 0x60, 120)),
            );
        } else {
            vgrad(&painter, inner, PANEL_TOP, PANEL_BOT);
        }
        painter.rect_stroke(cell.shrink(0.5), 2.0, Stroke::new(1.0, DIV_DARK));
        painter.text(
            cell.center(),
            Align2::CENTER_CENTER,
            name,
            FontId::proportional(9.0),
            if active { ACCENT } else { CREAM_DIM },
        );
        if let Some(p) = click {
            if cell.contains(p) && !active {
                *cur = *v;
                changed = true;
            }
        }
    }
    changed
}

// ── Pitch / mod wheel ──────────────────────────────────────────────────────

/// A tall vertical performance wheel (0..1) with ridge lines and a position
/// indicator. Returns `true` while being dragged.
fn draw_wheel(ui: &mut Ui, label: &str, value: &mut f32, centered: bool) -> bool {
    let (resp, painter) = ui.allocate_painter(vec2(34.0, 122.0), Sense::click_and_drag());
    let rect = resp.rect;
    let body = Rect::from_min_max(
        pos2(rect.left() + 5.0, rect.top() + 2.0),
        pos2(rect.right() - 5.0, rect.bottom() - 15.0),
    );

    // Recessed slot.
    painter.rect_filled(rect, 4.0, Color32::from_rgb(0x08, 0x08, 0x09));

    // Cylindrical shading (dark edges, lit centre).
    let mid = body.center().x;
    let lh = Rect::from_min_max(body.left_top(), pos2(mid, body.bottom()));
    let rh = Rect::from_min_max(pos2(mid, body.top()), body.right_bottom());
    hgrad(
        &painter,
        lh,
        Color32::from_rgb(0x1c, 0x1c, 0x1e),
        Color32::from_rgb(0x4a, 0x4a, 0x4e),
    );
    hgrad(
        &painter,
        rh,
        Color32::from_rgb(0x4a, 0x4a, 0x4e),
        Color32::from_rgb(0x1c, 0x1c, 0x1e),
    );

    // Horizontal ridge lines.
    let n = 9;
    for i in 1..n {
        let y = body.top() + body.height() * i as f32 / n as f32;
        painter.line_segment(
            [pos2(body.left() + 1.0, y), pos2(body.right() - 1.0, y)],
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 110)),
        );
    }

    // Position indicator.
    let vv = (*value).clamp(0.0, 1.0);
    let iy = body.bottom() - vv * body.height();
    painter.rect_filled(
        Rect::from_min_max(pos2(body.left(), iy - 1.5), pos2(body.right(), iy + 1.5)),
        0.0,
        ACCENT,
    );
    painter.rect_stroke(body, 5.0, Stroke::new(1.0, Color32::BLACK));

    painter.text(
        pos2(rect.center().x, rect.bottom() - 7.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        CREAM_DIM,
    );

    let mut changed = false;
    if resp.dragged() {
        let dy = resp.drag_delta().y;
        *value = ((*value) - dy / body.height()).clamp(0.0, 1.0);
        changed = true;
    }
    if resp.double_clicked() {
        *value = if centered { 0.5 } else { 0.0 };
        changed = true;
    }
    changed
}

// ── Section panel ──────────────────────────────────────────────────────────

/// A Minimoog "section": a white letter-spaced centered header with an engraved
/// underline, thin engraved vertical dividers on both edges, sitting on the
/// shared brushed-metal sheet.
fn panel(ui: &mut Ui, title: &str, add: impl FnOnce(&mut Ui)) {
    let inner = egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(9.0, 6.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(spaced(title))
                        .color(HEADER_TEXT)
                        .size(11.0)
                        .strong(),
                );
            });
            ui.add_space(1.0);
            // Engraved underline (dark + light hairline).
            let w = ui.available_width();
            let (r, p) = ui.allocate_painter(vec2(w, 3.0), Sense::hover());
            let y = r.rect.center().y;
            let (l, rt) = (r.rect.left() + 4.0, r.rect.right() - 4.0);
            p.line_segment([pos2(l, y), pos2(rt, y)], Stroke::new(1.0, DIV_DARK));
            p.line_segment(
                [pos2(l, y + 1.0), pos2(rt, y + 1.0)],
                Stroke::new(1.0, DIV_LIGHT),
            );
            ui.add_space(4.0);
            add(ui);
        });

    // Engraved vertical dividers on the section edges.
    let rect = inner.response.rect;
    let p = ui.painter();
    for x in [rect.left() + 1.0, rect.right() - 1.0] {
        p.line_segment(
            [pos2(x, rect.top() + 2.0), pos2(x, rect.bottom() - 2.0)],
            Stroke::new(1.0, DIV_DARK),
        );
        p.line_segment(
            [
                pos2(x + 1.0, rect.top() + 2.0),
                pos2(x + 1.0, rect.bottom() - 2.0),
            ],
            Stroke::new(1.0, DIV_LIGHT),
        );
    }
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

pub struct App {
    tx: EventSender,
    shared: Arc<SharedState>,
    presets: Vec<Patch>,
    patch: Patch,
    channel: u8,
    preset_idx: Option<usize>,
    // f32 shadows for the integer osc params so knob dragging stays smooth.
    osc_oct: [f32; 3],
    osc_semi: [f32; 3],
    // Remembered levels so the mixer ON/OFF rockers can restore them.
    osc_prev_level: [f32; 3],
    noise_prev_level: f32,
    // Spring-centered pitch wheel (visual only).
    pitch_wheel: f32,
    // virtual keyboard state
    kbd_octave: i32,
    active_notes: HashSet<i32>,
    pc_notes: HashMap<egui::Key, i32>,
    mouse_note: Option<i32>,
    // meters
    vu_db: f32,
    peak_db: f32,
    // real oak-wood photo texture for the side cheeks (None = procedural fallback)
    wood_tex: Option<egui::TextureHandle>,
    // Automated screenshot: when VOOG_SHOT=<path> is set, render a few frames,
    // grab the framebuffer to a PNG and quit. Used to generate docs images.
    shot_path: Option<String>,
    shot_frame: u32,
    // Owns the cpal output stream so it stays alive for the App's lifetime.
    // Used by the web build (which self-initializes audio); `None` natively,
    // where `main` owns the stream instead.
    _stream: Option<cpal::Stream>,
}

/// Decode the embedded oak-wood photo into a GPU texture (once).
fn load_wood(ctx: &Context) -> Option<egui::TextureHandle> {
    let bytes = include_bytes!("../assets/wood.jpg");
    let img = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    let color = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
    Some(ctx.load_texture("oak-wood", color, egui::TextureOptions::LINEAR))
}

impl App {
    fn new(
        cc: &eframe::CreationContext<'_>,
        tx: EventSender,
        shared: Arc<SharedState>,
        presets: Vec<Patch>,
    ) -> Self {
        Self::install_theme(&cc.egui_ctx);
        let wood_tex = load_wood(&cc.egui_ctx);
        let shot_path = std::env::var("VOOG_SHOT").ok();
        if shot_path.is_some() {
            // Screenshot mode: shrink the UI slightly so the entire panel fits the
            // capture window even on a short display (the normal app is unaffected).
            cc.egui_ctx.set_zoom_factor(0.8);
        }
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
            osc_prev_level: [1.0; 3],
            noise_prev_level: 0.5,
            pitch_wheel: 0.5,
            kbd_octave: 4,
            active_notes: HashSet::new(),
            pc_notes: HashMap::new(),
            mouse_note: None,
            vu_db: -60.0,
            peak_db: -60.0,
            wood_tex,
            shot_path,
            shot_frame: 0,
            _stream: None,
        };
        app.sync_shadows();
        app
    }

    /// Web constructor: self-initializes the whole engine the way `main` does
    /// natively — creates the `Synth`, the bounded event channel, the shared
    /// meter state, loads the factory presets and starts the cpal (Web Audio)
    /// output stream, storing it inside the `App` so it stays alive.
    ///
    /// Audio start is best-effort: browsers block the `AudioContext` until a
    /// user gesture, so a failure here must not prevent the UI from rendering.
    #[cfg(target_arch = "wasm32")]
    pub fn new_web(cc: &eframe::CreationContext<'_>) -> Self {
        use voog_dsp::{patches::factory_presets, Synth};

        let (tx, rx) = crossbeam_channel::bounded::<Event>(4096);
        let shared = Arc::new(SharedState::new());
        let synth = Synth::new();

        // Best-effort: keep the returned stream alive inside the App. If the
        // browser refuses to start audio (autoplay policy) we still render.
        let stream = crate::audio::start(synth, rx, shared.clone()).ok();

        let mut app = Self::new(cc, tx, shared, factory_presets());
        app._stream = stream;
        app
    }

    /// Web only: browsers keep the `AudioContext` suspended until a user gesture,
    /// and cpal only calls `resume()` once when the stream is built (at page load,
    /// too early). Re-issue `play()` on every fresh click / key press — that calls
    /// `AudioContext.resume()` again, which the browser now honours because a
    /// gesture has occurred, so audio actually starts flowing.
    #[cfg(target_arch = "wasm32")]
    fn kick_audio(&mut self, ctx: &Context) {
        let interacted = ctx.input(|i| {
            i.pointer.any_pressed()
                || i.events
                    .iter()
                    .any(|e| matches!(e, egui::Event::Key { pressed: true, .. }))
        });
        if interacted {
            if let Some(stream) = &self._stream {
                use cpal::traits::StreamTrait;
                let _ = stream.play();
            }
        }
    }

    /// When `VOOG_SHOT=<path>` is set: let the UI settle a few frames, request a
    /// framebuffer screenshot, save it as PNG and close. No OS screen-recording
    /// permission needed — it reads the app's own rendered image.
    fn handle_screenshot(&mut self, ctx: &Context) {
        if self.shot_path.is_none() {
            return;
        }
        self.shot_frame += 1;
        let shot = ctx.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(img) = shot {
            let [w, h] = img.size;
            let mut rgba = Vec::with_capacity(w * h * 4);
            for p in img.pixels.iter() {
                rgba.extend_from_slice(&p.to_array());
            }
            if let Some(path) = &self.shot_path {
                let _ = image::save_buffer(
                    path,
                    &rgba,
                    w as u32,
                    h as u32,
                    image::ExtendedColorType::Rgba8,
                );
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        } else if self.shot_frame == 12 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
        }
    }

    fn install_theme(ctx: &Context) {
        let mut v = egui::Visuals::dark();
        v.override_text_color = Some(CREAM);
        v.panel_fill = BG;
        v.window_fill = BG;
        v.faint_bg_color = PANEL_BG;
        v.extreme_bg_color = TROUGH;
        v.widgets.noninteractive.bg_fill = PANEL_BG;
        v.widgets.inactive.bg_fill = Color32::from_rgb(0x33, 0x36, 0x38);
        v.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x33, 0x36, 0x38);
        v.widgets.hovered.bg_fill = Color32::from_rgb(0x4a, 0x4e, 0x50);
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
            if o.level > 0.001 {
                self.osc_prev_level[k] = o.level;
            }
        }
        if self.patch.noise.level > 0.001 {
            self.noise_prev_level = self.patch.noise.level;
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
                    .color(HEADER_TEXT)
                    .size(24.0)
                    .strong(),
            );
            ui.add_space(10.0);
            ui.label(
                RichText::new("MODEL D  •  VIRTUAL ANALOG SYNTHESIZER")
                    .color(ACCENT)
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
                    Color32::from_rgb(0x33, 0x36, 0x38)
                })
                .min_size(vec2(30.0, 24.0));
                if ui.add(btn).clicked() {
                    self.set_channel(ch);
                }
            }
        });
    }

    fn body(&mut self, ui: &mut Ui) {
        // Matte anodized sheet behind every section, with faint vertical brush.
        let sheet = ui.max_rect();
        let painter = ui.painter();
        vgrad(painter, sheet, PANEL_TOP, PANEL_BOT);
        let mut x = sheet.left() + 7.0;
        while x < sheet.right() {
            painter.line_segment(
                [pos2(x, sheet.top()), pos2(x, sheet.bottom())],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 5)),
            );
            x += 13.0;
        }

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
            ui.label(RichText::new("WAVEFORM").color(CREAM_DIM).size(8.0));
            let mut wf = self.patch.oscillators[i].waveform;
            if segmented(ui, &mut wf, &WAVEFORMS) {
                self.patch.oscillators[i].waveform = wf;
                self.send(Event::SetOscWaveform {
                    channel: self.channel,
                    osc: i,
                    waveform: wf,
                });
            }
            ui.add_space(3.0);
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
                    if lvl > 0.001 {
                        self.osc_prev_level[i] = lvl;
                    }
                    self.set_param(ParamId::OscLevel(i), lvl);
                }
            });
            // Mixer enable paddle (routes this source into the mixer).
            let mut on = self.patch.oscillators[i].level > 0.001;
            if rocker(ui, "MIX", &mut on) {
                let value = if on {
                    let prev = self.osc_prev_level[i];
                    if prev > 0.001 {
                        prev
                    } else {
                        1.0
                    }
                } else {
                    self.osc_prev_level[i] = self.patch.oscillators[i].level;
                    0.0
                };
                self.set_param(ParamId::OscLevel(i), value);
            }
        });
    }

    fn noise_panel(&mut self, ui: &mut Ui) {
        panel(ui, "NOISE", |ui| {
            ui.label(RichText::new("TYPE").color(CREAM_DIM).size(8.0));
            let mut nt = self.patch.noise.noise_type;
            if segmented(ui, &mut nt, &NOISE_TYPES) {
                self.patch.noise.noise_type = nt;
                self.send(Event::SetNoiseType {
                    channel: self.channel,
                    noise_type: nt,
                });
            }
            ui.add_space(3.0);
            ui.horizontal(|ui| {
                let mut lvl = self.patch.noise.level;
                if knob(ui, "LEVEL", &mut lvl, 0.0, 1.0, false, 0.0, KFmt::Percent) {
                    if lvl > 0.001 {
                        self.noise_prev_level = lvl;
                    }
                    self.set_param(ParamId::NoiseLevel, lvl);
                }
            });
            let mut on = self.patch.noise.level > 0.001;
            if rocker(ui, "MIX", &mut on) {
                let value = if on {
                    if self.noise_prev_level > 0.001 {
                        self.noise_prev_level
                    } else {
                        0.5
                    }
                } else {
                    self.noise_prev_level = self.patch.noise.level;
                    0.0
                };
                self.set_param(ParamId::NoiseLevel, value);
            }
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
            ui.label(RichText::new("WAVEFORM").color(CREAM_DIM).size(8.0));
            let mut wf = self.patch.lfo.waveform;
            if segmented(ui, &mut wf, &WAVEFORMS) {
                self.patch.lfo.waveform = wf;
                self.send(Event::SetLfoWaveform {
                    channel: self.channel,
                    waveform: wf,
                });
            }
            ui.add_space(2.0);
            ui.label(RichText::new("DESTINATION").color(CREAM_DIM).size(8.0));
            let mut d = self.patch.lfo.destination;
            if segmented(ui, &mut d, &LFO_DESTS) {
                self.patch.lfo.destination = d;
                self.send(Event::SetLfoDest {
                    channel: self.channel,
                    dest: d,
                });
            }
            ui.add_space(3.0);
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
            ui.label(RichText::new("MODE").color(CREAM_DIM).size(8.0));
            let mut m = self.patch.glide.mode;
            if segmented(ui, &mut m, &GLIDE_MODES) {
                self.patch.glide.mode = m;
                self.send(Event::SetGlideMode {
                    channel: self.channel,
                    mode: m,
                });
            }
            ui.add_space(3.0);
            ui.horizontal(|ui| {
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
        painter.rect_filled(rect, 3.0, Color32::from_rgb(0x10, 0x10, 0x11));
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0, Color32::BLACK));
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

        ui.horizontal(|ui| {
            self.wheels(ui);
            ui.add_space(10.0);
            self.draw_keys(ui);
        });
    }

    /// Pitch + mod wheels on a black sub-panel to the left of the keys.
    fn wheels(&mut self, ui: &mut Ui) {
        egui::Frame::none()
            .fill(Color32::from_rgb(0x0b, 0x0b, 0x0c))
            .rounding(4.0)
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Spring-centered pitch wheel (visual only — sends nothing).
                    let mut p = self.pitch_wheel;
                    let dragging = draw_wheel(ui, "PITCH", &mut p, true);
                    self.pitch_wheel = if dragging {
                        p
                    } else {
                        // Ease back toward center when released.
                        self.pitch_wheel + (0.5 - self.pitch_wheel) * 0.25
                    };
                    ui.add_space(2.0);
                    // Mod wheel drives LFO depth on the selected channel.
                    let mut m = self.patch.lfo.depth;
                    if draw_wheel(ui, "MOD", &mut m, false) {
                        self.set_param(ParamId::LfoDepth, m);
                    }
                });
            });
    }

    fn draw_keys(&mut self, ui: &mut Ui) {
        let avail_w = ui.available_width();
        let (resp, painter) = ui.allocate_painter(vec2(avail_w, 130.0), Sense::click_and_drag());
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

        // Draw white keys — cream with a soft bottom shadow.
        let names = ["C", "", "D", "", "E", "F", "", "G", "", "A", "", "B"];
        for (r, note) in &white_notes {
            let on = self.active_notes.contains(note);
            let body = if on { ACCENT } else { WHITE_KEY };
            painter.rect(
                *r,
                3.0,
                body,
                Stroke::new(1.0, Color32::from_rgb(0x9a, 0x92, 0x7c)),
            );
            // Bottom shadow.
            let shadow = Rect::from_min_max(pos2(r.left(), r.bottom() - 14.0), r.max);
            vgrad(
                &painter,
                shadow,
                Color32::from_rgba_unmultiplied(0, 0, 0, 0),
                Color32::from_rgba_unmultiplied(0, 0, 0, if on { 30 } else { 55 }),
            );
            // Top highlight.
            painter.line_segment(
                [
                    pos2(r.left() + 2.0, r.top() + 1.5),
                    pos2(r.right() - 2.0, r.top() + 1.5),
                ],
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 90)),
            );
            let semi = (note.rem_euclid(12)) as usize;
            let label = if semi == 0 {
                format!("C{}", note / 12 - 1)
            } else {
                names[semi].to_string()
            };
            painter.text(
                pos2(r.center().x, r.bottom() - 10.0),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(8.0),
                CREAM_DIM,
            );
        }
        // Draw glossy black keys on top.
        for (r, note) in &black_notes {
            let on = self.active_notes.contains(note);
            let body = if on { ACCENT_DIM } else { BLACK_KEY };
            painter.rect(*r, 2.0, body, Stroke::new(1.0, Color32::BLACK));
            // Gloss highlight down the top of the key.
            let gloss = Rect::from_min_max(
                pos2(r.left() + 1.5, r.top() + 1.5),
                pos2(r.right() - 1.5, r.top() + r.height() * 0.55),
            );
            vgrad(
                &painter,
                gloss,
                Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, if on { 40 } else { 55 }),
                Color32::from_rgba_unmultiplied(0xff, 0xff, 0xff, 0),
            );
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
        #[cfg(target_arch = "wasm32")]
        self.kick_audio(ctx);
        self.process_keyboard(ctx);
        self.update_meters();

        egui::TopBottomPanel::top("header")
            .frame(egui::Frame::none().fill(PANEL_TOP).inner_margin(10.0))
            .show(ctx, |ui| self.header(ui));

        egui::TopBottomPanel::top("topbar")
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(0x1c, 0x1e, 0x1f))
                    .inner_margin(8.0),
            )
            .show(ctx, |ui| self.top_bar(ui));

        egui::TopBottomPanel::bottom("keyboard")
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(0x0e, 0x0e, 0x0f))
                    .inner_margin(10.0),
            )
            .show(ctx, |ui| self.keyboard(ui));

        // Oak wood side cheeks framing the metal panel (real photo texture).
        let wood = self.wood_tex.clone();
        egui::SidePanel::left("wood_left")
            .exact_width(26.0)
            .resizable(false)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let r = ui.max_rect();
                paint_wood(ui.painter(), r, wood.as_ref());
            });
        egui::SidePanel::right("wood_right")
            .exact_width(26.0)
            .resizable(false)
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let r = ui.max_rect();
                paint_wood(ui.painter(), r, wood.as_ref());
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(PANEL_BOT).inner_margin(8.0))
            .show(ctx, |ui| self.body(ui));

        self.handle_screenshot(ctx);

        // Native: repaint every frame for smooth meters (audio is on its own
        // thread). Web: throttle to ~30 fps so the main-thread audio scheduler
        // isn't starved by continuous canvas rendering (prevents crackle).
        #[cfg(target_arch = "wasm32")]
        ctx.request_repaint_after(std::time::Duration::from_millis(33));
        #[cfg(not(target_arch = "wasm32"))]
        ctx.request_repaint();
    }
}

/// Run the GUI on the main thread (blocks until the window closes). Native-only;
/// the web build boots the same `App` via `App::new_web` from `lib.rs`.
#[cfg(not(target_arch = "wasm32"))]
pub fn run(tx: EventSender, shared: Arc<SharedState>, presets: Vec<Patch>) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Rusted Moog")
            .with_inner_size([1080.0, 1010.0])
            .with_min_inner_size([860.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rusted Moog",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc, tx, shared, presets)))),
    )
}
