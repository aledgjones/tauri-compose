use super::components::measurements::PaddingMm;
use super::components::text::{Align, Font, FontFamily, Justify};
use super::layouts::LayoutType;
use crate::renderer::converter::{Mm, Space};
use crate::AppState;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri::State;

#[derive(Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BracketingApproach {
    None,
    Orchestral,
    SmallEnsemble,
}

#[derive(Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BracketStyle {
    None,
    Wing,
    Line,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Engraves {
    pub by_layout_type: FxHashMap<LayoutType, Engrave>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Engrave {
    pub space: Mm,

    pub frame_padding: PaddingMm,
    pub instrument_spacing: Space,
    pub stave_spacing: Space,
    pub system_start_padding: Space,

    pub instrument_name_font: Font,
    pub instrument_name_spacing: Space,

    pub systemic_barline_single_instrument_system: bool,
    pub bracketing_approach: BracketingApproach,
    pub bracket_style: BracketStyle,
    pub bracket_single_staves: bool,

    pub base_note_space: Space,
    pub minimum_note_space: Space,
    pub minimum_tie_space: Space,
    pub note_space_ratio: f32,

    pub maximum_beam_slant: f32,
}

impl Engraves {
    pub fn new() -> Self {
        let mut by_layout_type = FxHashMap::default();

        by_layout_type.insert(
            LayoutType::FullScore,
            Engrave {
                space: 2.0,

                frame_padding: PaddingMm::new(35.0, 25.0, 35.0, 25.0),
                instrument_spacing: 8.0,
                stave_spacing: 6.0,
                system_start_padding: 0.75,

                instrument_name_font: Font {
                    size: 1.75,
                    family: FontFamily::LibreBaskerville,
                    justify: Justify::End,
                    align: Align::Middle,
                },
                instrument_name_spacing: 2.0,

                systemic_barline_single_instrument_system: true,
                bracketing_approach: BracketingApproach::Orchestral,
                bracket_style: BracketStyle::Wing,
                bracket_single_staves: false,

                base_note_space: 3.0,
                minimum_note_space: 0.6,
                minimum_tie_space: 1.0,
                note_space_ratio: 1.41,

                maximum_beam_slant: 0.5,
            },
        );

        by_layout_type.insert(
            LayoutType::Part,
            Engrave {
                space: 2.0,

                frame_padding: PaddingMm::new(35.0, 25.0, 35.0, 25.0),
                instrument_spacing: 8.0,
                stave_spacing: 6.0,
                system_start_padding: 0.75,

                instrument_name_font: Font {
                    size: 1.75,
                    family: FontFamily::LibreBaskerville,
                    justify: Justify::End,
                    align: Align::Middle,
                },
                instrument_name_spacing: 2.0,

                systemic_barline_single_instrument_system: false,
                bracketing_approach: BracketingApproach::Orchestral,
                bracket_style: BracketStyle::Wing,
                bracket_single_staves: false,

                base_note_space: 3.0,
                minimum_note_space: 0.6,
                minimum_tie_space: 1.0,
                note_space_ratio: 1.41,

                maximum_beam_slant: 0.5,
            },
        );

        Self { by_layout_type }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_systemic_barline_single_instrument_system_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    layout_type: LayoutType,
    value: bool,
) {
    let mut state = state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.systemic_barline_single_instrument_system = value;

    state.emit(&app);
}

//     pub fn get_bracketing_approach(&self, key: &str) -> BracketingApproach {
//         let config = &self.file.score.engrave.by_key[key];
//         config.bracketing_approach
//     }

//     pub fn set_bracketing_approach(&mut self, key: &str, value: BracketingApproach) {
//         let config = self.file.score.engrave.by_key.get_mut(key).unwrap();
//         config.bracketing_approach = value;

//         self.emit();
//     }

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_bracket_style_set(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    layout_type: LayoutType,
    bracket_style: BracketStyle,
) {
    let mut state = state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.bracket_style = bracket_style;

    state.emit(&app);
}

//     pub fn get_bracket_single_staves(&self, key: &str) -> bool {
//         let config = &self.file.score.engrave.by_key[key];
//         config.bracket_single_staves
//     }

//     pub fn set_bracket_single_staves(&mut self, key: &str, value: bool) {
//         let config = self.file.score.engrave.by_key.get_mut(key).unwrap();
//         config.bracket_single_staves = value;

//         self.emit();
//     }

//     pub fn get_sub_bracket(&self, key: &str) -> bool {
//         let config = &self.file.score.engrave.by_key[key];
//         config.sub_bracket
//     }

//     pub fn set_sub_bracket(&mut self, key: &str, value: bool) {
//         let config = self.file.score.engrave.by_key.get_mut(key).unwrap();
//         config.sub_bracket = value;

//         self.emit();
//     }

//     pub fn get_base_note_space(&self, key: &str) -> f32 {
//         let config = &self.file.score.engrave.by_key[key];
//         config.base_note_space
//     }

//     pub fn set_base_note_space(&mut self, key: &str, value: f32) {
//         let config = self.file.score.engrave.by_key.get_mut(key).unwrap();
//         config.base_note_space = value;

//         self.emit();
//     }

//     pub fn get_minimum_note_space(&self, key: &str) -> f32 {
//         let config = &self.file.score.engrave.by_key[key];
//         config.minimum_note_space
//     }

//     pub fn set_minimum_note_space(&mut self, key: &str, value: f32) {
//         let config = self.file.score.engrave.by_key.get_mut(key).unwrap();
//         config.minimum_note_space = value;

//         self.emit();
//     }

//     pub fn get_minimum_tie_space(&self, key: &str) -> f32 {
//         let config = &self.file.score.engrave.by_key[key];
//         config.minimum_tie_space
//     }

//     pub fn set_minimum_tie_space(&mut self, key: &str, value: f32) {
//         let config = self.file.score.engrave.by_key.get_mut(key).unwrap();
//         config.minimum_tie_space = value;

//         self.emit();
//     }

//     pub fn get_note_space_ratio(&self, key: &str) -> f32 {
//         let config = &self.file.score.engrave.by_key[key];
//         config.note_space_ratio
//     }

//     pub fn set_note_space_ratio(&mut self, key: &str, value: f32) {
//         let config = self.file.score.engrave.by_key.get_mut(key).unwrap();
//         config.note_space_ratio = value;

//         self.emit();
//     }

//     pub fn get_space(&self, key: &str) -> f32 {
//         let config = &self.file.score.engrave.by_key[key];
//         config.space
//     }

//     pub fn set_space(&mut self, key: &str, value: f32) {
//         let config = self.file.score.engrave.by_key.get_mut(key).unwrap();
//         config.space = value;

//         self.emit();
//     }
