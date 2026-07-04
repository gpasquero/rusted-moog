//! Built-in factory presets. PORT OF `synth/patch/default_patches.py`.
//!
//! TEMPORARY minimal set — to be expanded to the full 19 presets.

use crate::params::Patch;

/// All factory presets, in display order. Each `Patch.name` is the preset name.
pub fn factory_presets() -> Vec<Patch> {
    vec![Patch { name: "Init".to_string(), ..Patch::default() }]
}
