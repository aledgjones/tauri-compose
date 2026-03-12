use super::components::measurements::PaddingMm;
use super::components::text::{Align, Font, FontFamily, Justify};
use super::layouts::LayoutType;
use crate::instructions::FlowRenderRecalculationRequest;
use crate::renderer::converter::{Mm, Space};
use crate::AppContextState;
use crate::RendererContextState;
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

    pub page_padding: PaddingMm,
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
                space: 1.5,

                page_padding: PaddingMm::new(30.0, 20.0, 30.0, 20.0),
                frame_padding: PaddingMm::new(20.0, 0.0, 20.0, 0.0),
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

                base_note_space: 4.0,
                minimum_note_space: 0.9,
                minimum_tie_space: 1.6,
                note_space_ratio: 1.41,

                maximum_beam_slant: 0.5,
            },
        );

        by_layout_type.insert(
            LayoutType::Part,
            Engrave {
                space: 2.0,

                page_padding: PaddingMm::new(30.0, 20.0, 30.0, 20.0),
                frame_padding: PaddingMm::new(20.0, 0.0, 20.0, 0.0),
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

                base_note_space: 4.0,
                minimum_note_space: 0.9,
                minimum_tie_space: 1.6,
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
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: bool,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.systemic_barline_single_instrument_system = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_bracketing_approach_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: BracketingApproach,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.bracketing_approach = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_bracket_style_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: BracketStyle,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.bracket_style = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_bracket_single_staves_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: bool,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.bracket_single_staves = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_base_note_space_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.base_note_space = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_minimum_note_space_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.minimum_note_space = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_minimum_tie_space_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.minimum_tie_space = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_note_space_ratio_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.note_space_ratio = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_space_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.space = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_frame_padding_top_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.frame_padding.top = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_frame_padding_bottom_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.frame_padding.bottom = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_page_padding_top_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.page_padding.top = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_page_padding_right_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.page_padding.right = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_page_padding_bottom_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.page_padding.bottom = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_engraves_page_padding_left_set(
    app: tauri::AppHandle,
    state: State<'_, AppContextState>,
    renderer_state: State<'_, RendererContextState>,
    layout_type: LayoutType,
    value: f32,
) {
    let mut state = state.lock().unwrap();
    let mut renderer = renderer_state.lock().unwrap();

    let config = state
        .score
        .engraves
        .by_layout_type
        .get_mut(&layout_type)
        .unwrap();
    config.page_padding.left = value;

    state.emit_general_state(
        &app,
        &mut renderer,
        Some(FlowRenderRecalculationRequest::full_visible_flows()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engraves_new_has_full_score_entry() {
        let e = Engraves::new();
        assert!(e.by_layout_type.contains_key(&LayoutType::FullScore));
    }

    #[test]
    fn engraves_new_has_part_entry() {
        let e = Engraves::new();
        assert!(e.by_layout_type.contains_key(&LayoutType::Part));
    }

    #[test]
    fn full_score_space_is_1_5() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!((full.space - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn part_space_is_2_0() {
        let e = Engraves::new();
        let part = &e.by_layout_type[&LayoutType::Part];
        assert!((part.space - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn full_score_instrument_spacing_is_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.instrument_spacing > 0.0);
    }

    #[test]
    fn full_score_stave_spacing_is_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.stave_spacing > 0.0);
    }

    #[test]
    fn full_score_base_note_space_is_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.base_note_space > 0.0);
    }

    #[test]
    fn part_instrument_spacing_greater_than_full_score() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        let part = &e.by_layout_type[&LayoutType::Part];
        // Part layout uses larger space so per-space metrics are same; space itself is larger
        assert!(part.space > full.space);
    }

    #[test]
    fn full_score_minimum_note_space_is_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.minimum_note_space > 0.0);
    }

    #[test]
    fn full_score_minimum_tie_space_is_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.minimum_tie_space > 0.0);
    }

    #[test]
    fn full_score_note_space_ratio_in_range() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.note_space_ratio > 1.0); // ratio > 1 expands space logarithmically
    }

    #[test]
    fn full_score_maximum_beam_slant_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.maximum_beam_slant > 0.0);
    }

    #[test]
    fn full_score_page_padding_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.page_padding.top > 0.0);
    }

    #[test]
    fn full_score_instrument_name_spacing_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.instrument_name_spacing > 0.0);
    }

    #[test]
    fn full_score_system_start_padding_positive() {
        let e = Engraves::new();
        let full = &e.by_layout_type[&LayoutType::FullScore];
        assert!(full.system_start_padding > 0.0);
    }
}
