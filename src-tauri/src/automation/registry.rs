use std::collections::HashSet;

use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

use crate::files::load_project_from_file;
use crate::playback::transport::TransportSharedState;
use crate::score::components::articulation::{Articulation, Articulations};
use crate::score::components::duration::NoteDuration;
use crate::score::components::pitch::Accidental;
use crate::score::config::{
    score_config_auto_count_styles_section_set, score_config_auto_count_styles_solo_set,
    AutoCountStyle,
};
use crate::score::engraves::{
    score_engraves_base_note_space_set, score_engraves_bracket_single_staves_set,
    score_engraves_bracket_style_set, score_engraves_bracketing_approach_set,
    score_engraves_frame_padding_bottom_set, score_engraves_frame_padding_top_set,
    score_engraves_minimum_note_space_set, score_engraves_minimum_tie_space_set,
    score_engraves_note_space_ratio_set, score_engraves_page_padding_bottom_set,
    score_engraves_page_padding_left_set, score_engraves_page_padding_right_set,
    score_engraves_page_padding_top_set, score_engraves_space_set,
    score_engraves_systemic_barline_single_instrument_system_set, BracketStyle, BracketingApproach,
};
use crate::score::entries::barline::{
    score_entries_barline_clear, score_entries_barline_create, Barline, BarlineDrawType,
};
use crate::score::entries::clef::{
    score_entries_clef_clear, score_entries_clef_create, score_entries_clef_update, ClefDrawType,
};
use crate::score::entries::key_signature::{
    score_entries_key_signature_clear, score_entries_key_signature_create,
    score_entries_key_signature_update, KeySignature, KeySignatureMode,
};
use crate::score::entries::tempo::{
    score_entries_tempo_clear, score_entries_tempo_create, score_entries_tempo_update, Tempo,
};
use crate::score::entries::time_signature::{
    score_entries_time_signature_clear, score_entries_time_signature_create,
    score_entries_time_signature_update, TimeSignature, TimeSignatureDrawType,
};
use crate::score::entries::tone::{
    score_entries_tone_accidental_set, score_entries_tone_articulation_set,
    score_entries_tone_create, score_entries_tone_duration_remove, score_entries_tone_duration_set,
    score_entries_tone_pitch_set, score_entries_tone_pitch_shift_by_octaves,
    score_entries_tone_pitch_shift_by_steps, score_entries_tone_set_position_and_duration,
    score_entries_tone_shift, score_entries_tone_slice, score_entries_tone_unify_articulations,
    Tone,
};
use crate::score::entries::tuplet::{
    score_entries_tuplet_create, score_entries_tuplet_remove, Tuplet, TupletEntry,
};
use crate::score::entries::Entry;
use crate::score::flows::{
    score_flows_assign_player, score_flows_create, score_flows_move, score_flows_remove,
    score_flows_rename, score_flows_unassign_player,
};
use crate::score::instruments::defs::score_instruments_get_all_definitions;
use crate::score::instruments::{
    score_instruments_assign_slot_route, score_instruments_create, score_instruments_move,
    score_instruments_remove, score_instruments_set_mute, score_instruments_set_pan,
    score_instruments_set_solo, score_instruments_set_volume, score_instruments_unassign_route,
    score_playback_slots_assign_plugin, score_playback_slots_create, score_playback_slots_remove,
    score_playback_slots_unassign_plugin,
};
use crate::score::layouts::{
    score_layouts_assign_flow, score_layouts_assign_player, score_layouts_create,
    score_layouts_move, score_layouts_remove, score_layouts_rename, score_layouts_unassign_flow,
    score_layouts_unassign_player, LayoutType,
};
use crate::score::meta::{
    score_meta_arranger_set, score_meta_artist_set, score_meta_composer_dates_set,
    score_meta_composer_set, score_meta_composition_year_set, score_meta_copyist_set,
    score_meta_copyright_set, score_meta_dedication_set, score_meta_editor_set,
    score_meta_lyricist_set, score_meta_other_information_set, score_meta_publisher_set,
    score_meta_subtitle_set, score_meta_title_set, score_meta_work_number_set,
};
use crate::score::players::{
    score_players_assign_instrument, score_players_create, score_players_move,
    score_players_remove, score_players_unassign_instrument, PlayerType,
};
use crate::utils::UniqueId;
use crate::{
    get_general_state_snapshot, reset_app_state, AppContextState, AudioContextState,
    RendererContextState,
};

#[derive(Clone, Serialize)]
pub struct ActionArgMetadata {
    name: &'static str,
    required: bool,
    description: &'static str,
}

#[derive(Clone, Serialize)]
pub struct ActionDefinition {
    name: &'static str,
    description: &'static str,
    args: Vec<ActionArgMetadata>,
    examples: Vec<Value>,
    prerequisites: Vec<Value>,
    #[serde(rename = "entityEffects")]
    entity_effects: Vec<Value>,
    returns: Vec<Value>,
}

#[derive(Serialize)]
struct LookupStaveTracks {
    stave_id: u64,
    master_track_id: u64,
    track_ids: Vec<u64>,
}

#[derive(Serialize)]
struct LookupInstrumentFlowTracks {
    flow_id: u64,
    player_id: u64,
    instrument_id: u64,
    definition_id: String,
    long_name: String,
    short_name: String,
    staves: Vec<LookupStaveTracks>,
}

#[derive(Serialize)]
struct LookupBar {
    bar_number: u64,
    start_tick: u64,
    length_ticks: u64,
    time_signature_id: u64,
}

pub fn action_definitions() -> Vec<ActionDefinition> {
    vec![
        action("reset", "Reset the current score back to a blank default state.", vec![]),
        action(
            "app.getActiveScore",
            "Inspect the active score path and currently selected UI ids.",
            vec![],
        ),
        action(
            "file.loadFromPath",
            "Load a project from a specific file path.",
            vec![arg("filePath", "Absolute path to a .soloc project file.")],
        ),
        action(
            "score.config.setSoloAutoCountStyle",
            "Set the auto-count style used for solo instruments. AutoCountStyle values: 0=Arabic, 1=Roman.",
            vec![arg("auto_count_style", "Numeric AutoCountStyle enum value. Use 0=Arabic or 1=Roman.")],
        ),
        action(
            "score.config.setSectionAutoCountStyle",
            "Set the auto-count style used for section instruments. AutoCountStyle values: 0=Arabic, 1=Roman.",
            vec![arg("auto_count_style", "Numeric AutoCountStyle enum value. Use 0=Arabic or 1=Roman.")],
        ),
        action(
            "score.engrave.setSystemicBarlineSingleInstrumentSystem",
            "Set whether single-instrument systems draw systemic barlines for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Whether to show systemic barlines on single-instrument systems."),
            ],
        ),
        action(
            "score.engrave.setBracketingApproach",
            "Set the bracketing approach for a layout type. BracketingApproach values: 0=None, 1=Orchestral, 2=SmallEnsemble.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Numeric BracketingApproach enum value. Use 0=None, 1=Orchestral, or 2=SmallEnsemble."),
            ],
        ),
        action(
            "score.engrave.setBracketStyle",
            "Set the bracket style for a layout type. BracketStyle values: 0=None, 1=Wing, 2=Line.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Numeric BracketStyle enum value. Use 0=None, 1=Wing, or 2=Line."),
            ],
        ),
        action(
            "score.engrave.setBracketSingleStaves",
            "Set whether single staves are bracketed for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Whether to bracket single staves."),
            ],
        ),
        action(
            "score.engrave.setBaseNoteSpace",
            "Set the base note spacing value for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point spacing value."),
            ],
        ),
        action(
            "score.engrave.setMinimumNoteSpace",
            "Set the minimum note spacing value for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point spacing value."),
            ],
        ),
        action(
            "score.engrave.setMinimumTieSpace",
            "Set the minimum tie spacing value for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point spacing value."),
            ],
        ),
        action(
            "score.engrave.setNoteSpaceRatio",
            "Set the note spacing ratio for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point ratio value."),
            ],
        ),
        action(
            "score.engrave.setSpace",
            "Set the base staff space size for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point space value."),
            ],
        ),
        action(
            "score.engrave.setFramePaddingTop",
            "Set the top frame padding for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point padding value in millimeters."),
            ],
        ),
        action(
            "score.engrave.setFramePaddingBottom",
            "Set the bottom frame padding for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point padding value in millimeters."),
            ],
        ),
        action(
            "score.engrave.setPagePaddingTop",
            "Set the top page padding for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point padding value in millimeters."),
            ],
        ),
        action(
            "score.engrave.setPagePaddingRight",
            "Set the right page padding for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point padding value in millimeters."),
            ],
        ),
        action(
            "score.engrave.setPagePaddingBottom",
            "Set the bottom page padding for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point padding value in millimeters."),
            ],
        ),
        action(
            "score.engrave.setPagePaddingLeft",
            "Set the left page padding for a layout type.",
            vec![
                arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part."),
                arg("value", "Floating-point padding value in millimeters."),
            ],
        ),
        action(
            "score.barline.create",
            "Create or replace a barline entry at a tick in a flow. BarlineDrawType values: 0=Single, 1=Double, 2=Triple, 3=Dashed, 4=Final, 5=StartRepeat, 6=EndRepeat, 7=EndStartRepeat.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("tick", "Tick position."),
                arg("barline_draw_type", "Numeric BarlineDrawType enum value."),
            ],
        ),
        action(
            "score.barline.clearAtTick",
            "Clear a barline entry at a tick in a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("tick", "Tick position.")],
        ),
        action(
            "score.clef.create",
            "Create or replace a clef entry on a stave master track. ClefDrawType values: 1=G, 2=F, 3=C, 4=Percussion.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("stave_id", "Numeric stave id."),
                arg("tick", "Tick position."),
                arg("pitch", "Pitch integer for the clef reference pitch."),
                arg("offset", "Visual offset from the middle stave line."),
                arg("clef_draw_type", "Numeric ClefDrawType enum value. Use 1=G, 2=F, 3=C, or 4=Percussion."),
                arg("hidden", "Whether the clef is hidden."),
            ],
        ),
        action(
            "score.clef.clearAtTick",
            "Clear a clef entry at a tick on a specific stave.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("stave_id", "Numeric stave id."),
                arg("tick", "Tick position."),
            ],
        ),
        action(
            "score.clef.update",
            "Update an existing clef entry.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("stave_id", "Numeric stave id."),
                arg("clef_id", "Numeric clef entry id."),
                arg("hidden", "Whether the clef is hidden."),
            ],
        ),
        action(
            "score.meta.setTitle",
            "Set the score title.",
            vec![arg("title", "Score title.")],
        ),
        action("score.meta.setSubtitle", "Set the score subtitle.", vec![arg("subtitle", "Score subtitle.")]),
        action("score.meta.setDedication", "Set the score dedication.", vec![arg("dedication", "Score dedication.")]),
        action("score.meta.setComposer", "Set the score composer.", vec![arg("composer", "Composer name.")]),
        action("score.meta.setArranger", "Set the score arranger.", vec![arg("arranger", "Arranger name.")]),
        action("score.meta.setLyricist", "Set the score lyricist.", vec![arg("lyricist", "Lyricist name.")]),
        action("score.meta.setArtist", "Set the score artist.", vec![arg("artist", "Artist name.")]),
        action("score.meta.setCopyist", "Set the score copyist.", vec![arg("copyist", "Copyist name.")]),
        action("score.meta.setPublisher", "Set the score publisher.", vec![arg("publisher", "Publisher name.")]),
        action("score.meta.setEditor", "Set the score editor.", vec![arg("editor", "Editor name.")]),
        action("score.meta.setCopyright", "Set the score copyright text.", vec![arg("copyright", "Copyright text.")]),
        action("score.meta.setWorkNumber", "Set the score work number.", vec![arg("work_number", "Work number string.")]),
        action("score.meta.setCompositionYear", "Set the composition year.", vec![arg("composition_year", "Composition year string.")]),
        action("score.meta.setComposerDates", "Set the composer dates text.", vec![arg("composer_dates", "Composer dates string.")]),
        action("score.meta.setOtherInformation", "Set the other information text.", vec![arg("other_information", "Free-form metadata text.")]),
        action("score.flow.create", "Create a new flow and return its numeric id.", vec![]),
        action(
            "score.flow.rename",
            "Rename an existing flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("title", "Flow title.")],
        ),
        action(
            "score.flow.move",
            "Reorder flows by moving one flow index to another.",
            vec![arg("from_index", "Zero-based source index."), arg("to_index", "Zero-based destination index.")],
        ),
        action(
            "score.flow.assignPlayer",
            "Assign a player to a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("player_id", "Numeric player id.")],
        ),
        action(
            "score.flow.unassignPlayer",
            "Unassign a player from a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("player_id", "Numeric player id.")],
        ),
        action("score.flow.remove", "Remove an existing flow.", vec![arg("flow_id", "Numeric flow id.")]),
        action(
            "score.player.create",
            "Create a new player. PlayerType values: 0=Solo, 1=Section.",
            vec![arg("player_type", "Numeric PlayerType enum value. Use 0=Solo or 1=Section.")],
        ),
        action(
            "score.player.move",
            "Reorder players by moving one player index to another.",
            vec![arg("from_index", "Zero-based source index."), arg("to_index", "Zero-based destination index.")],
        ),
        action(
            "score.player.assignInstrument",
            "Assign an existing instrument to an existing player.",
            vec![arg("player_id", "Numeric player id."), arg("instrument_id", "Numeric instrument id.")],
        ),
        action(
            "score.player.unassignInstrument",
            "Unassign an instrument from an existing player.",
            vec![arg("player_id", "Numeric player id."), arg("instrument_id", "Numeric instrument id.")],
        ),
        action("score.player.remove", "Remove an existing player.", vec![arg("player_id", "Numeric player id.")]),
        action(
            "score.instrument.create",
            "Create an instrument from a definition id such as keyboard.piano or strings.viola.",
            vec![arg("definition_id", "Stable instrument definition id.")],
        ),
        action(
            "score.instrument.getDefinitions",
            "List available instrument definition ids.",
            vec![],
        ),
        action(
            "score.instrument.move",
            "Move an instrument within or across players.",
            vec![
                arg("from_index", "Zero-based source instrument index."),
                arg("to_index", "Zero-based destination instrument index."),
                arg("from_player_id", "Numeric source player id."),
                arg("to_player_id", "Numeric destination player id."),
            ],
        ),
        action("score.instrument.remove", "Remove an instrument.", vec![arg("instrument_id", "Numeric instrument id.")]),
        action(
            "score.instrument.assignSlotRoute",
            "Assign an instrument route to an existing playback slot.",
            vec![
                arg("instrument_id", "Numeric instrument id."),
                arg(
                    "route",
                    "InstrumentPlaybackRoute payload with slot_id, target, and address.",
                ),
            ],
        ),
        action(
            "score.instrument.unassignRoute",
            "Unassign the playback route for an instrument.",
            vec![arg("instrument_id", "Numeric instrument id.")],
        ),
        action(
            "score.playback.slot.create",
            "Create an empty playback slot in the current score rack.",
            vec![],
        ),
        action(
            "score.playback.slot.remove",
            "Remove an existing playback slot from the current score rack.",
            vec![arg("slot_id", "Numeric playback slot id.")],
        ),
        action(
            "score.playback.slot.assignPlugin",
            "Assign a plugin to an existing playback slot.",
            vec![
                arg("slot_id", "Numeric playback slot id."),
                arg("plugin_id", "Plugin id string."),
            ],
        ),
        action(
            "score.playback.slot.unassignPlugin",
            "Unassign the current plugin from an existing playback slot.",
            vec![arg("slot_id", "Numeric playback slot id.")],
        ),
        action("score.instrument.setVolume", "Set an instrument volume value from 0-127.", vec![arg("instrument_id", "Numeric instrument id."), arg("volume", "Volume from 0-127.")]),
        action("score.instrument.setPan", "Set an instrument pan value from 0-127.", vec![arg("instrument_id", "Numeric instrument id."), arg("pan", "Pan from 0-127.")]),
        action("score.instrument.setMute", "Set an instrument mute state.", vec![arg("instrument_id", "Numeric instrument id."), arg("mute", "Whether the instrument is muted.")]),
        action("score.instrument.setSolo", "Set an instrument solo state.", vec![arg("instrument_id", "Numeric instrument id."), arg("solo", "Whether the instrument is soloed.")]),
        action(
            "score.layout.create",
            "Create a new layout. LayoutType values: 0=FullScore, 1=Part.",
            vec![arg("layout_type", "Numeric LayoutType enum value. Use 0=FullScore or 1=Part.")],
        ),
        action("score.layout.rename", "Rename an existing layout.", vec![arg("layout_id", "Numeric layout id."), arg("title", "Layout title.")]),
        action("score.layout.assignPlayer", "Assign a player to a layout.", vec![arg("layout_id", "Numeric layout id."), arg("player_id", "Numeric player id.")]),
        action("score.layout.unassignPlayer", "Unassign a player from a layout.", vec![arg("layout_id", "Numeric layout id."), arg("player_id", "Numeric player id.")]),
        action("score.layout.assignFlow", "Assign a flow to a layout.", vec![arg("layout_id", "Numeric layout id."), arg("flow_id", "Numeric flow id.")]),
        action("score.layout.unassignFlow", "Unassign a flow from a layout.", vec![arg("layout_id", "Numeric layout id."), arg("flow_id", "Numeric flow id.")]),
        action("score.layout.move", "Reorder layouts by moving one layout index to another.", vec![arg("from_index", "Zero-based source index."), arg("to_index", "Zero-based destination index.")]),
        action("score.layout.remove", "Remove an existing layout.", vec![arg("layout_id", "Numeric layout id.")]),
        action(
            "score.keySignature.clearAtTick",
            "Clear a key signature entry at a tick in a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("tick", "Tick position.")],
        ),
        action(
            "score.keySignature.create",
            "Create a key signature entry.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("tick", "Tick position."),
                arg("mode", "Numeric KeySignatureMode enum value. Use 0=Major or 1=Minor."),
                arg("offset", "Key signature accidental offset."),
                arg("hidden", "Whether the key signature is hidden."),
            ],
        ),
        action(
            "score.keySignature.update",
            "Update an existing key signature entry.",
            vec![arg("flow_id", "Numeric flow id."), arg("key_signature_id", "Numeric key signature entry id."), arg("hidden", "Whether the key signature is hidden.")],
        ),
        action(
            "score.timeSignature.clearAtTick",
            "Clear a time signature entry at a tick in a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("tick", "Tick position.")],
        ),
        action(
            "score.timeSignature.create",
            "Create a time signature entry.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("tick", "Tick position."),
                arg("beats", "Number of beats."),
                arg("beat_type", "Numeric NoteDuration enum value for the beat unit. Common values are 1=Whole, 2=Half, 4=Quarter, 8=Eighth, 16=Sixteenth, 32=ThirtySecond, 64=SixtyFourth, 128=HundredTwentyEighth."),
                arg("draw_type", "Numeric TimeSignatureDrawType enum value. Use 1=Regular, 2=CommonTime, 3=SplitCommonTime, 4=Open. Do not use 0; legacy hidden time signatures were migrated to hidden=true with draw_type=1."),
                arg("hidden", "Whether the entry is hidden."),
            ],
        ),
        action(
            "score.timeSignature.update",
            "Update an existing time signature entry.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("time_signature_id", "Numeric time signature entry id."),
                arg("draw_type", "Numeric TimeSignatureDrawType enum value. Use 1=Regular, 2=CommonTime, 3=SplitCommonTime, 4=Open."),
                arg("hidden", "Whether the entry is hidden."),
            ],
        ),
        action(
            "score.tempo.clearAtTick",
            "Clear a tempo entry at a tick in a flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("tick", "Tick position.")],
        ),
        action(
            "score.tempo.create",
            "Create a tempo entry on the flow master track.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("tick", "Tick position."),
                arg("text", "Tempo text such as Allegro, or an empty string."),
                arg("beat_type", "Numeric NoteDuration enum value for the metronome beat unit. Common values are 1=Whole, 2=Half, 4=Quarter, 8=Eighth, 16=Sixteenth, 32=ThirtySecond, 64=SixtyFourth, 128=HundredTwentyEighth."),
                arg("dots", "Number of augmentation dots on the metronome beat."),
                arg("bpm", "Tempo in beats per minute. Supported range is 20-400."),
                arg("hidden", "Whether the entry is hidden."),
                arg("show_text", "Whether to render the tempo text."),
                arg("show_metronome", "Whether to render the metronome marking."),
                arg("parenthesized", "Whether to wrap the metronome marking in parentheses."),
            ],
        ),
        action(
            "score.tempo.update",
            "Update an existing tempo entry in a flow.",
            vec![
                arg("flow_id", "Numeric flow id."),
                arg("tempo_id", "Numeric tempo entry id."),
                arg("text", "Tempo text such as Allegro, or an empty string."),
                arg("beat_type", "Numeric NoteDuration enum value for the metronome beat unit. Common values are 1=Whole, 2=Half, 4=Quarter, 8=Eighth, 16=Sixteenth, 32=ThirtySecond, 64=SixtyFourth, 128=HundredTwentyEighth."),
                arg("dots", "Number of augmentation dots on the metronome beat."),
                arg("bpm", "Tempo in beats per minute. Supported range is 20-400."),
                arg("hidden", "Whether the entry is hidden."),
                arg("show_text", "Whether to render the tempo text."),
                arg("show_metronome", "Whether to render the metronome marking."),
                arg("parenthesized", "Whether to wrap the metronome marking in parentheses."),
            ],
        ),
        action(
            "score.tone.create",
            "Create a tone entry on a track.",
            vec![
                arg("track_id", "Numeric track id."),
                arg("tick", "Tick position."),
                arg("duration", "Tone duration in ticks."),
                arg("pitch_int", "Chromatic pitch integer as a MIDI note number. Examples: C0=12, C4=60."),
                arg("accidental", "Accidental enum value or null. Use -2=DoubleFlat, -1=Flat, 0=Natural, 1=Sharp, 2=DoubleSharp."),
                arg("velocity", "Velocity from 0-127."),
                arg("articulations", "Array of Articulation enum values. Use 0=Staccato, 1=Tenuto, 2=Staccatissimo, 3=StaccatoTenuto, 4=Accent, 5=Marcato, 6=Stressed, 7=Unstressed."),
                optional_arg("tuplet_id", "Optional tuplet id for child insertion on the owning track."),
            ],
        ),
        action("score.tone.setDuration", "Set the duration of an existing tone.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("duration", "Tone duration in ticks."), optional_arg("tuplet_id", "Optional tuplet id used to address a nested tone directly on the owning track.")]),
        action("score.tone.slice", "Split a tone at a specific tick and return the new tone id when successful.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("slice_tick", "Tick position where the tone should be sliced in the target ownership context."), optional_arg("current_tuplet_id", "Optional tuplet id for the original owning context when slicing a tuplet-owned tone."), optional_arg("target_tuplet_id", "Optional tuplet id for the resulting tail tone when the split lands inside a tuplet context.")]),
        action("score.tone.setPitch", "Set the pitch of an existing tone.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("pitch_int", "Chromatic pitch integer as a MIDI note number. Examples: C0=12, C4=60."), arg("accidental", "Accidental enum value or null. Use -2=DoubleFlat, -1=Flat, 0=Natural, 1=Sharp, 2=DoubleSharp."), optional_arg("tuplet_id", "Optional tuplet id used to address a nested tone directly on the owning track.")]),
        action("score.tone.remove", "Remove an existing tone entry.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), optional_arg("tuplet_id", "Optional tuplet id used to address a nested tone directly on the owning track.")]),
        action("score.tone.setPositionAndDuration", "Set the position and duration of an existing tone.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("tick", "New tick position."), arg("duration", "New duration in ticks."), optional_arg("current_tuplet_id", "Optional tuplet id for the original owning context when moving a tuplet-owned tone."), optional_arg("target_tuplet_id", "Optional tuplet id for the resulting owning context when the moved tone lands inside a tuplet.")]),
        action("score.tone.shift", "Shift an existing tone entry to a new tick.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("new_tick", "New tick position."), optional_arg("current_tuplet_id", "Optional tuplet id for the original owning context when shifting a tuplet-owned tone."), optional_arg("target_tuplet_id", "Optional tuplet id for the resulting owning context when the shifted tone lands inside a tuplet.")]),
        action("score.tone.unifyArticulations", "Apply the articulation set from one tone to all tones at the same tick.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), optional_arg("tuplet_id", "Optional tuplet id used to address a nested tone directly on the owning track.")]),
        action("score.tone.shiftPitchBySteps", "Shift a tone pitch diatonically by a number of steps.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("steps", "Signed number of steps to shift."), optional_arg("tuplet_id", "Optional tuplet id used to address a nested tone directly on the owning track.")]),
        action("score.tone.shiftPitchByOctaves", "Shift a tone pitch by a number of octaves.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("octaves", "Signed number of octaves to shift."), optional_arg("tuplet_id", "Optional tuplet id used to address a nested tone directly on the owning track.")]),
        action("score.tone.setAccidental", "Set the accidental on a tone.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("accidental", "Accidental enum value. Use -2=DoubleFlat, -1=Flat, 0=Natural, 1=Sharp, 2=DoubleSharp."), optional_arg("tuplet_id", "Optional tuplet id used to address a nested tone directly on the owning track.")]),
        action("score.tone.toggleArticulation", "Toggle an articulation on a tone and apply the resulting articulation set across the tick.", vec![arg("track_id", "Numeric track id."), arg("tone_id", "Numeric tone id."), arg("articulation", "Numeric Articulation enum value. Use 0=Staccato, 1=Tenuto, 2=Staccatissimo, 3=StaccatoTenuto, 4=Accent, 5=Marcato, 6=Stressed, 7=Unstressed."), optional_arg("tuplet_id", "Optional tuplet id used to address a nested tone directly on the owning track.")]),
        action("score.tuplet.create", "Create a tuplet container on a track.", vec![arg("track_id", "Numeric track id."), arg("tick", "Tick position."), arg("note_count", "Number of notes in the tuplet group."), arg("in_space_of", "How many base notes the tuplet fits into."), arg("base_note_value", "Numeric NoteDuration enum value for the tuplet base note. Common values are 1=Whole, 2=Half, 4=Quarter, 8=Eighth, 16=Sixteenth, 32=ThirtySecond, 64=SixtyFourth, 128=HundredTwentyEighth."), arg("groupings", "Array of positive integer grouping sizes, such as [3] for a simple triplet.")]),
        action("score.tuplet.remove", "Remove a tuplet container and all of its child tones.", vec![arg("track_id", "Numeric track id."), arg("tuplet_id", "Numeric tuplet id.")]),
        action(
            "score.lookup.getInstrumentTracks",
            "Resolve the flow-local stave master ids and user track ids for an instrument in a specific flow.",
            vec![arg("flow_id", "Numeric flow id."), arg("instrument_id", "Numeric instrument id.")],
        ),
        action(
            "score.lookup.getScoreSlice",
            "Inspect a runtime score slice by optional flow, player, and instrument filters.",
            vec![
                optional_arg("flow_id", "Optional numeric flow id filter."),
                optional_arg("player_id", "Optional numeric player id filter."),
                optional_arg("instrument_id", "Optional numeric instrument id filter."),
            ],
        ),
        action(
            "score.lookup.getBarMap",
            "Inspect bar boundaries for a flow using its time-signature map.",
            vec![arg("flow_id", "Numeric flow id.")],
        ),
        action(
            "score.lookup.getFlowMarkers",
            "Inspect flow-level marker entries such as time signatures, key signatures, tempos, and barlines.",
            vec![arg("flow_id", "Numeric flow id.")],
        ),
        action(
            "score.lookup.getTrackEvents",
            "Inspect track-level tone and tuplet events within an optional tick range.",
            vec![
                arg("track_id", "Numeric track id."),
                optional_arg("start_tick", "Optional inclusive start tick. Defaults to 0."),
                optional_arg("end_tick", "Optional exclusive end tick. Defaults to the end of the track content."),
            ],
        ),
    ]
}

pub async fn execute_action(app: &AppHandle, name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "reset" => {
            reset_app_state(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<AudioContextState>(),
                app.state(),
            )?;
            Ok(Value::Null)
        }
        "app.getActiveScore" => Ok(derive_active_score(app)?),
        "file.loadFromPath" => {
            load_project_from_file(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<AudioContextState>(),
                app.state(),
                arg_string(args, 0, "filePath")?,
            )
            .await?;
            Ok(Value::Null)
        }
        "score.config.setSoloAutoCountStyle" => {
            score_config_auto_count_styles_solo_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<AutoCountStyle>(args, 0, "auto_count_style")?,
            );
            Ok(Value::Null)
        }
        "score.config.setSectionAutoCountStyle" => {
            score_config_auto_count_styles_section_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<AutoCountStyle>(args, 0, "auto_count_style")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setSystemicBarlineSingleInstrumentSystem" => {
            score_engraves_systemic_barline_single_instrument_system_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_bool(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setBracketingApproach" => {
            score_engraves_bracketing_approach_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_enum::<BracketingApproach>(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setBracketStyle" => {
            score_engraves_bracket_style_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_enum::<BracketStyle>(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setBracketSingleStaves" => {
            score_engraves_bracket_single_staves_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_bool(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setBaseNoteSpace" => {
            score_engraves_base_note_space_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setMinimumNoteSpace" => {
            score_engraves_minimum_note_space_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setMinimumTieSpace" => {
            score_engraves_minimum_tie_space_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setNoteSpaceRatio" => {
            score_engraves_note_space_ratio_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setSpace" => {
            score_engraves_space_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setFramePaddingTop" => {
            score_engraves_frame_padding_top_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setFramePaddingBottom" => {
            score_engraves_frame_padding_bottom_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setPagePaddingTop" => {
            score_engraves_page_padding_top_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setPagePaddingRight" => {
            score_engraves_page_padding_right_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setPagePaddingBottom" => {
            score_engraves_page_padding_bottom_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.engrave.setPagePaddingLeft" => {
            score_engraves_page_padding_left_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_enum::<LayoutType>(args, 0, "layout_type")?,
                arg_f32(args, 1, "value")?,
            );
            Ok(Value::Null)
        }
        "score.barline.create" => Ok(json!(score_entries_barline_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "flow_id")?,
            arg_u64(args, 1, "tick")?,
            arg_enum::<BarlineDrawType>(args, 2, "barline_draw_type")?,
        ))),
        "score.barline.clearAtTick" => {
            score_entries_barline_clear(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_u64(args, 1, "tick")?,
            );
            Ok(Value::Null)
        }
        "score.clef.create" => Ok(json!(score_entries_clef_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "flow_id")?,
            arg_id(args, 1, "stave_id")?,
            arg_u64(args, 2, "tick")?,
            arg_u8(args, 3, "pitch")?,
            arg_i8(args, 4, "offset")?,
            arg_enum::<ClefDrawType>(args, 5, "clef_draw_type")?,
            arg_bool(args, 6, "hidden")?,
        ))),
        "score.clef.clearAtTick" => {
            score_entries_clef_clear(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_id(args, 1, "stave_id")?,
                arg_u64(args, 2, "tick")?,
            );
            Ok(Value::Null)
        }
        "score.clef.update" => {
            score_entries_clef_update(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_id(args, 1, "stave_id")?,
                arg_id(args, 2, "clef_id")?,
                arg_bool(args, 3, "hidden")?,
            )?;
            Ok(Value::Null)
        }
        "score.meta.setTitle" => {
            score_meta_title_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "title")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setSubtitle" => {
            score_meta_subtitle_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "subtitle")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setDedication" => {
            score_meta_dedication_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "dedication")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setComposer" => {
            score_meta_composer_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "composer")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setArranger" => {
            score_meta_arranger_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "arranger")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setLyricist" => {
            score_meta_lyricist_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "lyricist")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setArtist" => {
            score_meta_artist_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "artist")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setCopyist" => {
            score_meta_copyist_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "copyist")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setPublisher" => {
            score_meta_publisher_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "publisher")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setEditor" => {
            score_meta_editor_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "editor")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setCopyright" => {
            score_meta_copyright_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "copyright")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setWorkNumber" => {
            score_meta_work_number_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "work_number")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setCompositionYear" => {
            score_meta_composition_year_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "composition_year")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setComposerDates" => {
            score_meta_composer_dates_set(
                app.clone(),
                app.state::<RendererContextState>(),
                app.state::<AppContextState>(),
                arg_string(args, 0, "composer_dates")?,
            );
            Ok(Value::Null)
        }
        "score.meta.setOtherInformation" => {
            score_meta_other_information_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_string(args, 0, "other_information")?,
            );
            Ok(Value::Null)
        }
        "score.flow.create" => Ok(json!(score_flows_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
        ))),
        "score.flow.rename" => {
            score_flows_rename(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_string(args, 1, "title")?,
            );
            Ok(Value::Null)
        }
        "score.flow.move" => {
            score_flows_move(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_usize(args, 0, "from_index")?,
                arg_usize(args, 1, "to_index")?,
            );
            Ok(Value::Null)
        }
        "score.flow.assignPlayer" => {
            score_flows_assign_player(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_id(args, 1, "player_id")?,
            );
            Ok(Value::Null)
        }
        "score.flow.unassignPlayer" => {
            score_flows_unassign_player(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_id(args, 1, "player_id")?,
            );
            Ok(Value::Null)
        }
        "score.flow.remove" => {
            score_flows_remove(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
            );
            Ok(Value::Null)
        }
        "score.player.create" => Ok(json!(score_players_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_enum::<PlayerType>(args, 0, "player_type")?,
        ))),
        "score.player.move" => {
            score_players_move(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_usize(args, 0, "from_index")?,
                arg_usize(args, 1, "to_index")?,
            );
            Ok(Value::Null)
        }
        "score.player.assignInstrument" => {
            score_players_assign_instrument(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "player_id")?,
                arg_id(args, 1, "instrument_id")?,
            );
            Ok(Value::Null)
        }
        "score.player.unassignInstrument" => {
            score_players_unassign_instrument(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "player_id")?,
                arg_id(args, 1, "instrument_id")?,
            );
            Ok(Value::Null)
        }
        "score.player.remove" => {
            score_players_remove(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "player_id")?,
            );
            Ok(Value::Null)
        }
        "score.instrument.create" => Ok(json!(score_instruments_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_string(args, 0, "definition_id")?,
        ))),
        "score.instrument.getDefinitions" => Ok(json!(score_instruments_get_all_definitions())),
        "score.instrument.move" => {
            score_instruments_move(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_usize(args, 0, "from_index")?,
                arg_usize(args, 1, "to_index")?,
                arg_id(args, 2, "from_player_id")?,
                arg_id(args, 3, "to_player_id")?,
            );
            Ok(Value::Null)
        }
        "score.instrument.remove" => {
            score_instruments_remove(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "instrument_id")?,
            );
            Ok(Value::Null)
        }
        "score.instrument.assignSlotRoute" => {
            score_instruments_assign_slot_route(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "instrument_id")?,
                arg_enum(args, 1, "route")?,
            )?;
            Ok(Value::Null)
        }
        "score.instrument.unassignRoute" => {
            score_instruments_unassign_route(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "instrument_id")?,
            )?;
            Ok(Value::Null)
        }
        "score.playback.slot.create" => {
            let slot_id = score_playback_slots_create(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
            )?;
            Ok(json!(slot_id))
        }
        "score.playback.slot.remove" => {
            score_playback_slots_remove(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "slot_id")?,
            )?;
            Ok(Value::Null)
        }
        "score.playback.slot.assignPlugin" => {
            score_playback_slots_assign_plugin(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "slot_id")?,
                arg_string(args, 1, "plugin_id")?,
            )?;
            Ok(Value::Null)
        }
        "score.playback.slot.unassignPlugin" => {
            score_playback_slots_unassign_plugin(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "slot_id")?,
            )?;
            Ok(Value::Null)
        }
        "score.instrument.setVolume" => {
            score_instruments_set_volume(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<TransportSharedState>(),
                arg_id(args, 0, "instrument_id")?,
                arg_u8(args, 1, "volume")?,
            );
            Ok(Value::Null)
        }
        "score.instrument.setPan" => {
            score_instruments_set_pan(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<TransportSharedState>(),
                arg_id(args, 0, "instrument_id")?,
                arg_u8(args, 1, "pan")?,
            );
            Ok(Value::Null)
        }
        "score.instrument.setMute" => {
            score_instruments_set_mute(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<TransportSharedState>(),
                arg_id(args, 0, "instrument_id")?,
                arg_bool(args, 1, "mute")?,
            );
            Ok(Value::Null)
        }
        "score.instrument.setSolo" => {
            score_instruments_set_solo(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<TransportSharedState>(),
                arg_id(args, 0, "instrument_id")?,
                arg_bool(args, 1, "solo")?,
            );
            Ok(Value::Null)
        }
        "score.layout.create" => Ok(json!(score_layouts_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_enum::<LayoutType>(args, 0, "layout_type")?,
        ))),
        "score.layout.rename" => {
            score_layouts_rename(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "layout_id")?,
                arg_string(args, 1, "title")?,
            );
            Ok(Value::Null)
        }
        "score.layout.assignPlayer" => {
            score_layouts_assign_player(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "layout_id")?,
                arg_id(args, 1, "player_id")?,
            );
            Ok(Value::Null)
        }
        "score.layout.unassignPlayer" => {
            score_layouts_unassign_player(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "layout_id")?,
                arg_id(args, 1, "player_id")?,
            );
            Ok(Value::Null)
        }
        "score.layout.assignFlow" => {
            score_layouts_assign_flow(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "layout_id")?,
                arg_id(args, 1, "flow_id")?,
            );
            Ok(Value::Null)
        }
        "score.layout.unassignFlow" => {
            score_layouts_unassign_flow(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "layout_id")?,
                arg_id(args, 1, "flow_id")?,
            );
            Ok(Value::Null)
        }
        "score.layout.move" => {
            score_layouts_move(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_usize(args, 0, "from_index")?,
                arg_usize(args, 1, "to_index")?,
            );
            Ok(Value::Null)
        }
        "score.layout.remove" => {
            score_layouts_remove(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "layout_id")?,
            );
            Ok(Value::Null)
        }
        "score.keySignature.clearAtTick" => {
            score_entries_key_signature_clear(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_u64(args, 1, "tick")?,
            );
            Ok(Value::Null)
        }
        "score.keySignature.create" => Ok(json!(score_entries_key_signature_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "flow_id")?,
            arg_u64(args, 1, "tick")?,
            arg_enum::<KeySignatureMode>(args, 2, "mode")?,
            arg_i8(args, 3, "offset")?,
            arg_bool(args, 4, "hidden")?,
        ))),
        "score.keySignature.update" => {
            score_entries_key_signature_update(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_id(args, 1, "key_signature_id")?,
                arg_bool(args, 2, "hidden")?,
            )?;
            Ok(Value::Null)
        }
        "score.timeSignature.clearAtTick" => {
            score_entries_time_signature_clear(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_u64(args, 1, "tick")?,
            );
            Ok(Value::Null)
        }
        "score.timeSignature.create" => Ok(json!(score_entries_time_signature_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "flow_id")?,
            arg_u64(args, 1, "tick")?,
            arg_u8(args, 2, "beats")?,
            arg_enum::<NoteDuration>(args, 3, "beat_type")?,
            arg_enum::<TimeSignatureDrawType>(args, 4, "draw_type")?,
            arg_bool(args, 5, "hidden")?,
        ))),
        "score.timeSignature.update" => {
            score_entries_time_signature_update(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "flow_id")?,
                arg_id(args, 1, "time_signature_id")?,
                arg_enum::<TimeSignatureDrawType>(args, 2, "draw_type")?,
                arg_bool(args, 3, "hidden")?,
            )?;
            Ok(Value::Null)
        }
        "score.tempo.clearAtTick" => {
            score_entries_tempo_clear(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<TransportSharedState>(),
                arg_id(args, 0, "flow_id")?,
                arg_u64(args, 1, "tick")?,
            );
            Ok(Value::Null)
        }
        "score.tempo.create" => Ok(json!(score_entries_tempo_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            app.state::<TransportSharedState>(),
            arg_id(args, 0, "flow_id")?,
            arg_u64(args, 1, "tick")?,
            arg_string(args, 2, "text")?,
            arg_enum::<NoteDuration>(args, 3, "beat_type")?,
            arg_u8(args, 4, "dots")?,
            arg_u16(args, 5, "bpm")?,
            arg_bool(args, 6, "hidden")?,
            arg_bool(args, 7, "show_text")?,
            arg_bool(args, 8, "show_metronome")?,
            arg_bool(args, 9, "parenthesized")?,
        ))),
        "score.tempo.update" => {
            score_entries_tempo_update(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                app.state::<TransportSharedState>(),
                arg_id(args, 0, "flow_id")?,
                arg_id(args, 1, "tempo_id")?,
                arg_string(args, 2, "text")?,
                arg_enum::<NoteDuration>(args, 3, "beat_type")?,
                arg_u8(args, 4, "dots")?,
                arg_u16(args, 5, "bpm")?,
                arg_bool(args, 6, "hidden")?,
                arg_bool(args, 7, "show_text")?,
                arg_bool(args, 8, "show_metronome")?,
                arg_bool(args, 9, "parenthesized")?,
            )?;
            Ok(Value::Null)
        }
        "score.tone.create" => Ok(json!(score_entries_tone_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "track_id")?,
            arg_u64(args, 1, "tick")?,
            arg_u64(args, 2, "duration")?,
            arg_u8(args, 3, "pitch_int")?,
            arg_optional_enum::<Accidental>(args, 4)?,
            arg_u8(args, 5, "velocity")?,
            arg_articulations(args, 6)?,
            arg_optional_id(args, 7)?,
        )?)),
        "score.tone.setDuration" => {
            score_entries_tone_duration_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "track_id")?,
                arg_id(args, 1, "tone_id")?,
                arg_u64(args, 2, "duration")?,
                arg_optional_id(args, 3)?,
            );
            Ok(Value::Null)
        }
        "score.tone.slice" => Ok(json!(score_entries_tone_slice(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "track_id")?,
            arg_id(args, 1, "tone_id")?,
            arg_u64(args, 2, "slice_tick")?,
            arg_optional_id(args, 3)?,
            arg_optional_id(args, 4)?,
        )?)),
        "score.tone.setPitch" => {
            score_entries_tone_pitch_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "track_id")?,
                arg_id(args, 1, "tone_id")?,
                arg_u8(args, 2, "pitch_int")?,
                arg_optional_enum::<Accidental>(args, 3)?,
                arg_optional_id(args, 4)?,
            );
            Ok(Value::Null)
        }
        "score.tone.remove" => {
            score_entries_tone_duration_remove(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "track_id")?,
                arg_id(args, 1, "tone_id")?,
                arg_optional_id(args, 2)?,
            );
            Ok(Value::Null)
        }
        "score.tuplet.remove" => {
            score_entries_tuplet_remove(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "track_id")?,
                arg_id(args, 1, "tuplet_id")?,
            )?;
            Ok(Value::Null)
        }
        "score.tone.setPositionAndDuration" => {
            score_entries_tone_set_position_and_duration(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "track_id")?,
                arg_id(args, 1, "tone_id")?,
                arg_u64(args, 2, "tick")?,
                arg_u64(args, 3, "duration")?,
                arg_optional_id(args, 4)?,
                arg_optional_id(args, 5)?,
            );
            Ok(Value::Null)
        }
        "score.tone.shift" => {
            score_entries_tone_shift(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "track_id")?,
                arg_id(args, 1, "tone_id")?,
                arg_u64(args, 2, "new_tick")?,
                arg_optional_id(args, 3)?,
                arg_optional_id(args, 4)?,
            );
            Ok(Value::Null)
        }
        "score.tone.unifyArticulations" => {
            score_entries_tone_unify_articulations(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "track_id")?,
                arg_id(args, 1, "tone_id")?,
                arg_optional_id(args, 2)?,
            );
            Ok(Value::Null)
        }
        "score.tone.shiftPitchBySteps" => Ok(json!(score_entries_tone_pitch_shift_by_steps(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "track_id")?,
            arg_id(args, 1, "tone_id")?,
            arg_i8(args, 2, "steps")?,
            arg_optional_id(args, 3)?,
        ))),
        "score.tone.shiftPitchByOctaves" => Ok(json!(score_entries_tone_pitch_shift_by_octaves(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "track_id")?,
            arg_id(args, 1, "tone_id")?,
            arg_i8(args, 2, "octaves")?,
            arg_optional_id(args, 3)?,
        ))),
        "score.tone.setAccidental" => Ok(json!(score_entries_tone_accidental_set(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "track_id")?,
            arg_id(args, 1, "tone_id")?,
            arg_enum::<Accidental>(args, 2, "accidental")?,
            arg_optional_id(args, 3)?,
        ))),
        "score.tone.toggleArticulation" => {
            score_entries_tone_articulation_set(
                app.clone(),
                app.state::<AppContextState>(),
                app.state::<RendererContextState>(),
                arg_id(args, 0, "track_id")?,
                arg_id(args, 1, "tone_id")?,
                arg_enum::<Articulation>(args, 2, "articulation")?,
                arg_optional_id(args, 3)?,
            );
            Ok(Value::Null)
        }
        "score.tuplet.create" => Ok(json!(score_entries_tuplet_create(
            app.clone(),
            app.state::<AppContextState>(),
            app.state::<RendererContextState>(),
            arg_id(args, 0, "track_id")?,
            arg_u64(args, 1, "tick")?,
            arg_u8(args, 2, "note_count")?,
            arg_u8(args, 3, "in_space_of")?,
            arg_enum::<NoteDuration>(args, 4, "base_note_value")?,
            arg_u8_vec(args, 5, "groupings")?,
        )?)),
        "score.lookup.getInstrumentTracks" => Ok(json!(derive_instrument_tracks(
            app,
            arg_id(args, 0, "flow_id")?,
            arg_id(args, 1, "instrument_id")?,
        )?)),
        "score.lookup.getScoreSlice" => Ok(derive_score_slice(
            app,
            optional_id(args, 0)?,
            optional_id(args, 1)?,
            optional_id(args, 2)?,
        )?),
        "score.lookup.getBarMap" => Ok(derive_bar_map(app, arg_id(args, 0, "flow_id")?)?),
        "score.lookup.getFlowMarkers" => Ok(derive_flow_markers(app, arg_id(args, 0, "flow_id")?)?),
        "score.lookup.getTrackEvents" => Ok(derive_track_events(
            app,
            arg_id(args, 0, "track_id")?,
            optional_u64(args, 1)?,
            optional_u64(args, 2)?,
        )?),
        _ => Err(format!("Unknown automation action: {name}")),
    }
}

fn action(
    name: &'static str,
    description: &'static str,
    args: Vec<ActionArgMetadata>,
) -> ActionDefinition {
    ActionDefinition {
        name,
        description,
        args,
        examples: vec![],
        prerequisites: vec![],
        entity_effects: vec![],
        returns: vec![],
    }
}

fn arg(name: &'static str, description: &'static str) -> ActionArgMetadata {
    ActionArgMetadata {
        name,
        required: true,
        description,
    }
}

fn optional_arg(name: &'static str, description: &'static str) -> ActionArgMetadata {
    ActionArgMetadata {
        name,
        required: false,
        description,
    }
}

fn derive_instrument_tracks(
    app: &AppHandle,
    flow_id: u64,
    instrument_id: u64,
) -> Result<LookupInstrumentFlowTracks, String> {
    let state = get_general_state_snapshot(app);
    let flow = state
        .score
        .flows
        .by_id
        .get(&flow_id)
        .ok_or_else(|| format!("Flow {flow_id} not found"))?;
    let instrument = state
        .score
        .instruments
        .by_id
        .get(&instrument_id)
        .ok_or_else(|| format!("Instrument {instrument_id} not found"))?;

    let player_id = state
        .score
        .players
        .order
        .iter()
        .copied()
        .find(|current_player_id| {
            state
                .score
                .players
                .by_id
                .get(current_player_id)
                .is_some_and(|player| player.instrument_ids.contains(&instrument_id))
        })
        .ok_or_else(|| {
            format!("Instrument {instrument_id} is not currently assigned to any player")
        })?;

    if !flow.player_ids.contains(&player_id) {
        return Err(format!(
            "Instrument {instrument_id} is owned by player {player_id} but that player is not assigned to flow {flow_id}"
        ));
    }

    let staves = instrument
        .stave_ids
        .iter()
        .copied()
        .map(|stave_id| {
            let stave = flow.staves.by_id.get(&stave_id).ok_or_else(|| {
                format!("Flow {flow_id} is missing stave {stave_id} for instrument {instrument_id}")
            })?;

            Ok(LookupStaveTracks {
                stave_id,
                master_track_id: stave.master_id,
                track_ids: stave.track_ids.clone(),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(LookupInstrumentFlowTracks {
        flow_id,
        player_id,
        instrument_id,
        definition_id: instrument.definition_id.clone(),
        long_name: instrument.long_name.clone(),
        short_name: instrument.short_name.clone(),
        staves,
    })
}

fn derive_score_slice(
    app: &AppHandle,
    flow_id: Option<u64>,
    player_id: Option<u64>,
    instrument_id: Option<u64>,
) -> Result<Value, String> {
    let state = get_general_state_snapshot(app);

    if let Some(id) = flow_id {
        if !state.score.flows.by_id.contains_key(&id) {
            return Err(format!("Flow {id} not found"));
        }
    }
    if let Some(id) = player_id {
        if !state.score.players.by_id.contains_key(&id) {
            return Err(format!("Player {id} not found"));
        }
    }
    if let Some(id) = instrument_id {
        if !state.score.instruments.by_id.contains_key(&id) {
            return Err(format!("Instrument {id} not found"));
        }
    }

    let player_ids = player_id
        .map(|id| vec![id])
        .unwrap_or_else(|| state.score.players.order.clone());

    let mut owned_instrument_ids = Vec::new();
    for current_player_id in &state.score.players.order {
        if let Some(player) = state.score.players.by_id.get(current_player_id) {
            for current_instrument_id in &player.instrument_ids {
                if !owned_instrument_ids.contains(current_instrument_id) {
                    owned_instrument_ids.push(*current_instrument_id);
                }
            }
        }
    }

    let orphaned_instrument_ids = state
        .score
        .instruments
        .by_id
        .keys()
        .copied()
        .filter(|id| !owned_instrument_ids.contains(id))
        .collect::<Vec<_>>();

    let instrument_ids = instrument_id.map(|id| vec![id]).unwrap_or_else(|| {
        let mut ids = owned_instrument_ids.clone();
        ids.extend(orphaned_instrument_ids);
        ids
    });
    let flow_ids = flow_id
        .map(|id| vec![id])
        .unwrap_or_else(|| state.score.flows.order.clone());

    let players = player_ids
        .iter()
        .map(|current_player_id| {
            let player = &state.score.players.by_id[current_player_id];
            json!({
                "id": player.id,
                "player_type": player.player_type,
                "instrument_ids": player.instrument_ids,
            })
        })
        .collect::<Vec<_>>();
    let instruments = instrument_ids
        .iter()
        .map(|current_instrument_id| {
            let instrument = &state.score.instruments.by_id[current_instrument_id];
            let resolved_playback_route = state
                .resolved_playback_route_by_instrument_id
                .get(current_instrument_id)
                .cloned();
            json!({
                "id": instrument.id,
                "definition_id": instrument.definition_id,
                "long_name": instrument.long_name,
                "short_name": instrument.short_name,
                "stave_ids": instrument.stave_ids,
                "playback_route": instrument.playback_route,
                "resolved_playback_route": resolved_playback_route,
            })
        })
        .collect::<Vec<_>>();
    let flows = flow_ids
        .iter()
        .map(|current_flow_id| {
            let flow = &state.score.flows.by_id[current_flow_id];
            let ordered_player_ids = state
                .score
                .players
                .order
                .iter()
                .copied()
                .filter(|current_player_id| flow.player_ids.contains(current_player_id))
                .collect::<Vec<_>>();

            json!({
                "id": flow.id,
                "title": flow.title,
                "master_track_id": flow.master_id,
                "player_ids": ordered_player_ids,
            })
        })
        .collect::<Vec<_>>();

    let allowed_player_ids = player_ids.iter().copied().collect::<HashSet<_>>();
    let mut flow_instruments = Vec::new();
    for current_flow_id in &flow_ids {
        for current_instrument_id in &instrument_ids {
            if let Ok(tracks) =
                derive_instrument_tracks(app, *current_flow_id, *current_instrument_id)
            {
                if allowed_player_ids.contains(&tracks.player_id) {
                    flow_instruments.push(json!(tracks));
                }
            }
        }
    }

    Ok(json!({
        "players": players,
        "instruments": instruments,
        "flows": flows,
        "flow_instruments": flow_instruments,
    }))
}

fn derive_active_score(app: &AppHandle) -> Result<Value, String> {
    let state = get_general_state_snapshot(app);

    Ok(json!({
        "file_path": state.current_file_path,
        "active_flow_id": state.ui.flow_id,
        "active_layout_id": state.ui.layout_id,
        "flow_ids": state.score.flows.order,
        "layout_ids": state.score.layouts.order,
    }))
}

fn derive_bar_map(app: &AppHandle, flow_id: u64) -> Result<Value, String> {
    let state = get_general_state_snapshot(app);
    let flow = state
        .score
        .flows
        .by_id
        .get(&flow_id)
        .ok_or_else(|| format!("Flow {flow_id} not found"))?;
    let master = state
        .score
        .tracks
        .by_id
        .get(&flow.master_id)
        .ok_or_else(|| format!("Flow {flow_id} master track {} not found", flow.master_id))?;

    let time_signatures = collect_time_signatures(master);
    if time_signatures.is_empty() {
        return Err(format!(
            "Flow {flow_id} has no time signatures on master track {}",
            flow.master_id
        ));
    }

    let mut bars = Vec::new();
    for (index, time_signature) in time_signatures.iter().enumerate() {
        let segment_end = time_signatures
            .get(index + 1)
            .map(|next| next.tick)
            .unwrap_or(flow.length);
        let mut tick = time_signature.tick;

        while tick < segment_end && tick < flow.length {
            let remaining_in_segment = segment_end.saturating_sub(tick);
            let remaining_in_flow = flow.length.saturating_sub(tick);
            let length_ticks = time_signature
                .ticks_per_bar()
                .min(remaining_in_segment)
                .min(remaining_in_flow);

            if length_ticks == 0 {
                break;
            }

            bars.push(LookupBar {
                bar_number: bars.len() as u64 + 1,
                start_tick: tick,
                length_ticks,
                time_signature_id: time_signature.id,
            });

            tick += length_ticks;
        }
    }

    Ok(json!({
        "flow_id": flow.id,
        "flow_length_ticks": flow.length,
        "bars": bars,
    }))
}

fn derive_flow_markers(app: &AppHandle, flow_id: u64) -> Result<Value, String> {
    let state = get_general_state_snapshot(app);
    let flow = state
        .score
        .flows
        .by_id
        .get(&flow_id)
        .ok_or_else(|| format!("Flow {flow_id} not found"))?;
    let master = state
        .score
        .tracks
        .by_id
        .get(&flow.master_id)
        .ok_or_else(|| format!("Flow {flow_id} master track {} not found", flow.master_id))?;

    Ok(json!({
        "flow_id": flow.id,
        "master_track_id": flow.master_id,
        "flow_length_ticks": flow.length,
        "barlines": collect_barlines(master).into_iter().map(serialize_barline).collect::<Vec<_>>(),
        "key_signatures": collect_key_signatures(master).into_iter().map(serialize_key_signature).collect::<Vec<_>>(),
        "time_signatures": collect_time_signatures(master).into_iter().map(serialize_time_signature).collect::<Vec<_>>(),
        "tempo_entries": collect_tempos(master).into_iter().map(serialize_tempo).collect::<Vec<_>>(),
    }))
}

fn derive_track_events(
    app: &AppHandle,
    track_id: u64,
    start_tick: Option<u64>,
    end_tick: Option<u64>,
) -> Result<Value, String> {
    let state = get_general_state_snapshot(app);
    let track = state
        .score
        .tracks
        .by_id
        .get(&track_id)
        .ok_or_else(|| format!("Track {track_id} not found"))?;
    let start_tick = start_tick.unwrap_or(0);
    let end_tick = end_tick.unwrap_or_else(|| max_track_tick(track).unwrap_or(start_tick));

    if end_tick < start_tick {
        return Err(format!(
            "end_tick {end_tick} must be greater than or equal to start_tick {start_tick}"
        ));
    }

    let tones = collect_tones(track)
        .into_iter()
        .filter(|tone| tick_in_half_open_range(tone.tick, start_tick, end_tick))
        .map(serialize_tone)
        .collect::<Vec<_>>();
    let tuplets = collect_tuplets(track)
        .into_iter()
        .filter(|tuplet| tick_in_half_open_range(tuplet.tick, start_tick, end_tick))
        .map(serialize_tuplet)
        .collect::<Vec<_>>();

    Ok(json!({
        "track_id": track.id,
        "start_tick": start_tick,
        "end_tick": end_tick,
        "tones": tones,
        "tuplets": tuplets,
    }))
}

fn collect_barlines(track: &crate::score::tracks::Track) -> Vec<&Barline> {
    let mut barlines = track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::Barline(barline) => Some(barline),
            _ => None,
        })
        .collect::<Vec<_>>();
    barlines.sort_by_key(|barline| (barline.tick, barline.id));
    barlines
}

fn collect_key_signatures(track: &crate::score::tracks::Track) -> Vec<&KeySignature> {
    let mut key_signatures = track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::KeySignature(key_signature) => Some(key_signature),
            _ => None,
        })
        .collect::<Vec<_>>();
    key_signatures.sort_by_key(|key_signature| (key_signature.tick, key_signature.id));
    key_signatures
}

fn collect_time_signatures(track: &crate::score::tracks::Track) -> Vec<&TimeSignature> {
    let mut time_signatures = track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::TimeSignature(time_signature) => Some(time_signature),
            _ => None,
        })
        .collect::<Vec<_>>();
    time_signatures.sort_by_key(|time_signature| (time_signature.tick, time_signature.id));
    time_signatures
}

fn collect_tempos(track: &crate::score::tracks::Track) -> Vec<&Tempo> {
    let mut tempos = track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::Tempo(tempo) => Some(tempo),
            _ => None,
        })
        .collect::<Vec<_>>();
    tempos.sort_by_key(|tempo| (tempo.tick, tempo.id));
    tempos
}

fn collect_tones(track: &crate::score::tracks::Track) -> Vec<&Tone> {
    let mut tones = track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::Tone(tone) => Some(tone),
            _ => None,
        })
        .collect::<Vec<_>>();
    tones.sort_by_key(|tone| (tone.tick, tone.id));
    tones
}

fn collect_tuplets(track: &crate::score::tracks::Track) -> Vec<&Tuplet> {
    let mut tuplets = track
        .entries
        .by_id
        .values()
        .filter_map(|entry| match entry {
            Entry::Tuplet(tuplet) => Some(tuplet.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    tuplets.sort_by_key(|tuplet| (tuplet.tick, tuplet.id));
    tuplets
}

fn serialize_barline(barline: &Barline) -> Value {
    json!({
        "barline_id": barline.id,
        "tick": barline.tick,
        "barline_draw_type": barline.barline_draw_type,
    })
}

fn serialize_key_signature(key_signature: &KeySignature) -> Value {
    json!({
        "key_signature_id": key_signature.id,
        "tick": key_signature.tick,
        "mode": key_signature.mode,
        "offset": key_signature.offset,
        "hidden": key_signature.hidden,
    })
}

fn serialize_time_signature(time_signature: &TimeSignature) -> Value {
    json!({
        "time_signature_id": time_signature.id,
        "tick": time_signature.tick,
        "beats": time_signature.beats,
        "beat_type": time_signature.beat_type,
        "draw_type": time_signature.draw_type,
        "hidden": time_signature.hidden,
        "groupings": time_signature.groupings,
        "ticks_per_bar": time_signature.ticks_per_bar(),
    })
}

fn serialize_tempo(tempo: &Tempo) -> Value {
    json!({
        "tempo_id": tempo.id,
        "tick": tempo.tick,
        "text": tempo.text,
        "beat_type": tempo.beat_type,
        "dots": tempo.dots,
        "bpm": tempo.bpm,
        "hidden": tempo.hidden,
        "show_text": tempo.show_text,
        "show_metronome": tempo.show_metronome,
        "parenthesized": tempo.parenthesized,
    })
}

fn serialize_tone(tone: &Tone) -> Value {
    json!({
        "tone_id": tone.id,
        "tick": tone.tick,
        "duration": tone.duration,
        "pitch_int": tone.pitch.int,
        "accidental": tone.pitch.accidental,
        "velocity": tone.velocity.int(),
        "articulations": tone.articulations,
    })
}

fn serialize_tuplet(tuplet: &Tuplet) -> Value {
    let mut tones = tuplet
        .entries
        .by_id
        .values()
        .map(|entry| match entry {
            TupletEntry::Tone(tone) => tone,
        })
        .collect::<Vec<_>>();
    tones.sort_by_key(|tone| (tone.tick, tone.id));

    json!({
        "tuplet_id": tuplet.id,
        "tick": tuplet.tick,
        "note_count": tuplet.note_count,
        "in_space_of": tuplet.in_space_of,
        "base_note_value": tuplet.base_note_value,
        "groupings": tuplet.groupings,
        "consumed_ticks": tuplet.consumed_ticks(),
        "local_length_ticks": tuplet.local_length_ticks(),
        "tones": tones.into_iter().map(serialize_tone).collect::<Vec<_>>(),
    })
}

fn max_track_tick(track: &crate::score::tracks::Track) -> Option<u64> {
    let tone_end = collect_tones(track)
        .into_iter()
        .map(|tone| tone.tick + tone.duration)
        .max();
    let tuplet_end = collect_tuplets(track)
        .into_iter()
        .map(|tuplet| tuplet.tick + tuplet.consumed_ticks())
        .max();

    tone_end.into_iter().chain(tuplet_end).max()
}

fn tick_in_half_open_range(tick: u64, start_tick: u64, end_tick: u64) -> bool {
    tick >= start_tick && tick < end_tick
}

fn arg_value<'a>(args: &'a [Value], index: usize, name: &str) -> Result<&'a Value, String> {
    args.get(index)
        .ok_or_else(|| format!("Missing argument {name} at position {index}"))
}

fn arg_string(args: &[Value], index: usize, name: &str) -> Result<String, String> {
    arg_value(args, index, name)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| format!("Argument {name} must be a string"))
}

fn arg_id(args: &[Value], index: usize, name: &str) -> Result<u64, String> {
    arg_value(args, index, name)?
        .as_u64()
        .ok_or_else(|| format!("Argument {name} must be a positive integer"))
}

fn optional_id(args: &[Value], index: usize) -> Result<Option<u64>, String> {
    match args.get(index) {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            format!("Argument at position {index} must be a positive integer or null")
        }),
    }
}

fn optional_u64(args: &[Value], index: usize) -> Result<Option<u64>, String> {
    optional_id(args, index)
}

fn arg_u64(args: &[Value], index: usize, name: &str) -> Result<u64, String> {
    arg_id(args, index, name)
}

fn arg_u8(args: &[Value], index: usize, name: &str) -> Result<u8, String> {
    let value = arg_id(args, index, name)?;
    u8::try_from(value).map_err(|_| format!("Argument {name} is out of range for u8"))
}

fn arg_u16(args: &[Value], index: usize, name: &str) -> Result<u16, String> {
    let value = arg_id(args, index, name)?;
    u16::try_from(value).map_err(|_| format!("Argument {name} is out of range for u16"))
}

fn arg_usize(args: &[Value], index: usize, name: &str) -> Result<usize, String> {
    let value = arg_id(args, index, name)?;
    usize::try_from(value).map_err(|_| format!("Argument {name} is out of range for usize"))
}

fn arg_i8(args: &[Value], index: usize, name: &str) -> Result<i8, String> {
    let value = arg_value(args, index, name)?
        .as_i64()
        .ok_or_else(|| format!("Argument {name} must be an integer"))?;
    i8::try_from(value).map_err(|_| format!("Argument {name} is out of range for i8"))
}

fn arg_f32(args: &[Value], index: usize, name: &str) -> Result<f32, String> {
    let value = arg_value(args, index, name)?
        .as_f64()
        .ok_or_else(|| format!("Argument {name} must be a number"))?;
    Ok(value as f32)
}

fn arg_bool(args: &[Value], index: usize, name: &str) -> Result<bool, String> {
    arg_value(args, index, name)?
        .as_bool()
        .ok_or_else(|| format!("Argument {name} must be a boolean"))
}

fn arg_optional_id(args: &[Value], index: usize) -> Result<Option<UniqueId>, String> {
    match args.get(index) {
        Some(Value::Number(number)) => number
            .as_u64()
            .map(Some)
            .ok_or_else(|| format!("arg {index} must be a positive integer")),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(format!("arg {index} must be a positive integer or null")),
    }
}

fn arg_enum<T>(args: &[Value], index: usize, name: &str) -> Result<T, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    serde_json::from_value(arg_value(args, index, name)?.clone())
        .map_err(|error| format!("Argument {name} could not be parsed: {error}"))
}

fn arg_optional_enum<T>(args: &[Value], index: usize) -> Result<Option<T>, String>
where
    T: for<'de> serde::Deserialize<'de>,
{
    match args.get(index) {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(|error| format!("Argument at position {index} could not be parsed: {error}")),
    }
}

fn arg_articulations(args: &[Value], index: usize) -> Result<Articulations, String> {
    serde_json::from_value(arg_value(args, index, "articulations")?.clone())
        .map_err(|error| format!("Argument articulations could not be parsed: {error}"))
}

fn arg_u8_vec(args: &[Value], index: usize, name: &str) -> Result<Vec<u8>, String> {
    serde_json::from_value(arg_value(args, index, name)?.clone())
        .map_err(|error| format!("Argument {name} could not be parsed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::action_definitions;

    #[test]
    fn action_definitions_include_tuplet_create() {
        let definition = action_definitions()
            .into_iter()
            .find(|definition| definition.name == "score.tuplet.create")
            .expect("expected score.tuplet.create action definition");

        let arg_names = definition
            .args
            .into_iter()
            .map(|arg| arg.name)
            .collect::<Vec<_>>();

        assert_eq!(
            arg_names,
            vec![
                "track_id",
                "tick",
                "note_count",
                "in_space_of",
                "base_note_value",
                "groupings",
            ]
        );
    }

    #[test]
    fn action_definitions_include_new_lookup_tools() {
        let names = action_definitions()
            .into_iter()
            .map(|definition| definition.name)
            .collect::<Vec<_>>();

        assert!(names.contains(&"app.getActiveScore"));
        assert!(names.contains(&"score.lookup.getBarMap"));
        assert!(names.contains(&"score.lookup.getFlowMarkers"));
        assert!(names.contains(&"score.lookup.getTrackEvents"));
    }
}
