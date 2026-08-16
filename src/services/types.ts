export type UniqueId = number;

export type Tick = number;
export type Ticks = number;

export enum BarlineDrawType {
  Single = 0,
  Double,
  Triple,
  Dashed,
  Final,
  StartRepeat,
  EndRepeat,
  EndStartRepeat,
}

export enum ClefDrawType {
  G = 1,
  F = 2,
  C = 3,
  Percussion = 4,
}

export enum NoteDuration {
  Whole = 1,
  Half = 2,
  Quarter = 4,
  Eighth = 8,
  Sixteenth = 16,
  ThirtySecond = 32,
  SixtyFourth = 64,
  HudredTwentyEighth = 128,
}

export enum Accidental {
  DoubleSharp = 2,
  Sharp = 1,
  Natural = 0,
  Flat = -1,
  DoubleFlat = -2,
}

export enum Articulation {
  Staccato,
  Tenuto,
  Staccatissimo,
  StaccatoTenuto,
  Accent,
  Marcato,
  Stressed,
  Unstressed,
}

export enum Tool {
  Select,
  Write,
  Slice,
  Erase,
}

export enum Tab {
  Setup,
  Write,
  Engrave,
  Sequence,
  Print,
}

export enum SetupSelectionType {
  Player,
  Flow,
  Layout,
  Tone,
}

export enum PlayerType {
  Solo,
  Section,
}

export enum AutoCountStyle {
  Arabic,
  Roman,
}

export enum BracketingApproach {
  None,
  Orchestral,
  SmallEnsemble,
}

export enum BracketStyle {
  None,
  Wing,
  Line,
}

export type Meta = {
  app_version: string;
  title: string;
  subtitle: string;
  dedication: string;
  composer: string;
  arranger: string;
  lyricist: string;
  artist: string;
  copyist: string;
  publisher: string;
  editor: string;
  copyright: string;
  work_number: string;
  composition_year: string;
  composer_dates: string;
  other_information: string;
  created: number;
};

export type Config = {
  auto_count_styles: {
    solo: AutoCountStyle;
    section: AutoCountStyle;
  };
};

export enum InstrumentType {
  Melodic = 0,
  Percussive,
}

export enum PlaybackRouteTargetMode {
  Plugin = 0,
  PluginPart,
  MidiChannel,
}

export type PlaybackRouteTarget = {
  mode: PlaybackRouteTargetMode;
  index: number | null;
};

export enum PlaybackRouteAddressMode {
  Root = 0,
  Part,
  MidiChannel,
}

export type ExpressionMapCondition =
  | {
      id: string;
      type: "no_op";
    }
  | {
      id: string;
      type: "articulation";
      comparator: ExpressionMapEqualityComparator;
      articulation: Articulation | null;
    }
  | {
      id: string;
      type: "playing_technique";
      comparator: ExpressionMapEqualityComparator;
      playing_technique: PlayingTechniqueKind | null;
    }
  | {
      id: string;
      type: "written_note_length";
      comparator: ExpressionMapNoteLengthComparator;
      note_duration: NoteDuration;
    };

export enum ExpressionMapEqualityComparator {
  Equals = "equals",
  NotEquals = "not_equals",
}

export enum ExpressionMapNoteLengthComparator {
  LessThan = "less_than",
  LessThanOrEqual = "less_than_or_equal",
  Exactly = "exactly",
  NotEqual = "not_equal",
  GreaterThanOrEqual = "greater_than_or_equal",
  GreaterThan = "greater_than",
}

export type ExpressionMapAction =
  | {
      id: string;
      type: "keyswitch_note";
      midi_note: number;
      velocity: number;
    }
  | {
      id: string;
      type: "control_change";
      controller: number;
      value: number;
    }
  | {
      id: string;
      type: "no_op";
    }
  | {
      id: string;
      type: "note_length_transform";
      ratio_percent: number;
    };

export type ExpressionMapRule = {
  id: string;
  name: string;
  conditions: ExpressionMapCondition[];
  actions: ExpressionMapAction[];
};

export type ExpressionMap = {
  id: string;
  name: string;
  creator: string;
  version: number;
  default_actions: ExpressionMapAction[];
  rules: ExpressionMapRule[];
};

export type ExpressionMapLibraryState = {
  order: string[];
  by_id: Record<string, ExpressionMap>;
};

export type ExpressionMapExportPayload = {
  version: 1;
  expression_maps: ExpressionMap[];
};

export enum ResolvedExpressionMapLinkStatus {
  Unassigned = 0,
  Assigned,
}

export type ResolvedExpressionMapLink = {
  status: ResolvedExpressionMapLinkStatus;
  expression_map_id: string | null;
  expression_map_name: string | null;
};

export type PlaybackRouteAddress = {
  mode: PlaybackRouteAddressMode;
  index: number | null;
};

export type InstrumentPlaybackRoute = {
  slot_id: UniqueId;
  target: PlaybackRouteTarget;
  address: PlaybackRouteAddress;
  expression_map_id?: string | null;
};

export type PlaybackRouteChoice = {
  key: string;
  label: string;
  target: PlaybackRouteTarget;
  address: PlaybackRouteAddress;
};

export type PlaybackSlotTarget = {
  plugin_id: string;
  plugin_state?: PlaybackSlotPluginState | null;
};

export type PlaybackSlotPluginState = {
  vst3_component_state_base64: string;
};

export type PlaybackRackSlot = {
  id: UniqueId;
  target: PlaybackSlotTarget | null;
};

export type PlaybackRack = {
  order: UniqueId[];
  by_id: Record<UniqueId, PlaybackRackSlot>;
};

export type ScorePlayback = {
  rack: PlaybackRack;
};

export enum ResolvedPlaybackRouteStatus {
  Unrouted = 0,
  Ready,
  NeedsAttention,
}

export type SupportedPlaybackMixControls = {
  volume: boolean;
  pan: boolean;
  mute: boolean;
  solo: boolean;
};

export type ResolvedPlaybackRoute = {
  status: ResolvedPlaybackRouteStatus;
  slot_id: UniqueId | null;
  slot_target: PlaybackSlotTarget | null;
  effective_plugin_id: string | null;
  target: PlaybackRouteTarget | null;
  address: PlaybackRouteAddress | null;
  supported_mix_controls?: SupportedPlaybackMixControls;
};

export type Instrument = {
  id: UniqueId;
  definition_id: string;
  instrument_type: InstrumentType;
  long_name: string;
  short_name: string;
  stave_ids: UniqueId[];
  count: string | null;
  playback_route?: InstrumentPlaybackRoute | null;
  volume: number;
  pan: number;
  mute: boolean;
  solo: boolean;
};

export type Instruments = {
  by_id: Record<UniqueId, Instrument>;
};

export type Player = {
  id: UniqueId;
  player_type: PlayerType;
  instrument_ids: UniqueId[];
};

export type Players = {
  order: UniqueId[];
  by_id: Record<UniqueId, Player>;
};

export type Pitch = {
  int: number;
  accidental: Accidental;
};

export type Velocity = {
  int: number;
};

export enum KeySignatureMode {
  Major,
  Minor,
}

export enum PlayingTechniqueKind {
  Pizz,
  Arco,
}

export enum TimeSignatureDrawType {
  Regular = 1, // 4/4 etc
  CommonTime = 2, // C
  SplitCommonTime = 3, // ¢
  Open = 4, // X
}

export type Barline = {
  id: UniqueId;
  tick: Tick;
  type: "Barline";
  barline_draw_type: BarlineDrawType;
};

export type Clef = {
  id: UniqueId;
  tick: Tick;
  type: "Clef";
  hidden: boolean;
  draw_as: ClefDrawType;
  pitch: Pitch;
  offset: number;
};

export type TimeSignature = {
  id: UniqueId;
  tick: Tick;
  type: "TimeSignature";
  hidden: boolean;
  beats: number;
  beat_type: NoteDuration;
  draw_type: TimeSignatureDrawType;
};

export type Tempo = {
  id: UniqueId;
  tick: Tick;
  type: "Tempo";
  text: string;
  beat_type: NoteDuration;
  dots: number;
  bpm: number;
  hidden: boolean;
  show_text: boolean;
  show_metronome: boolean;
  parenthesized: boolean;
};

export type KeySignature = {
  id: UniqueId;
  tick: Tick;
  type: "KeySignature";
  hidden: boolean;
  mode: KeySignatureMode;
  offset: number;
};

export type PlayingTechnique = {
  id: UniqueId;
  tick: Tick;
  type: "PlayingTechnique";
  hidden: boolean;
  kind: PlayingTechniqueKind;
};

export type Tone = {
  id: UniqueId;
  tick: Tick;
  type: "Tone";
  duration: Ticks;
  pitch: Pitch;
  velocity: Velocity;
  articulations: Articulation[];
};

export type Entries<T> = {
  by_tick: Record<Tick, UniqueId[]>;
  by_id: Record<UniqueId, T>;
};

export type TupletEntry = Tone;

export type Tuplet = {
  id: UniqueId;
  tick: Tick;
  type: "Tuplet";
  note_count: number;
  in_space_of: number;
  base_note_value: NoteDuration;
  groupings: number[];
  entries: Entries<TupletEntry>;
};

export type Entry =
  | Barline
  | Clef
  | TimeSignature
  | KeySignature
  | PlayingTechnique
  | Tone
  | Tempo
  | Tuplet;

export type Track = {
  id: UniqueId;
  entries: Entries<Entry>;
};

export type Tracks = {
  by_id: Record<UniqueId, Track>;
};

export type Stave = {
  id: UniqueId;
  lines: Array<0 | 1>;
  master_id: UniqueId;
  track_ids: UniqueId[];
};

type Staves = {
  by_id: Record<UniqueId, Stave>;
};

export type Flow = {
  id: UniqueId;
  title: string;
  player_ids: UniqueId[];
  length: number;
  master_id: UniqueId;
  staves: Staves;
};

export type Flows = {
  order: UniqueId[];
  by_id: Record<UniqueId, Flow>;
};

export enum LayoutType {
  FullScore = 0,
  Part,
}

export type Layout = {
  id: UniqueId;
  title: string;
  layout_type: LayoutType;
  player_ids: UniqueId[];
  flow_ids: UniqueId[];
};

export type Layouts = {
  order: UniqueId[];
  by_id: Record<UniqueId, Layout>;
};

export enum FontFamily {
  Bravura,
  LibreBaskerville,
}

export enum Justify {
  Start,
  Middle,
  End,
}

export enum Align {
  Top,
  Middle,
  Bottom,
}

export type Font = {
  size: number;
  family: FontFamily;
  justify: Justify;
  align: Align;
};

export type Padding = {
  top: number;
  right: number;
  bottom: number;
  left: number;
};

export type Engrave = {
  space: number;

  page_padding: Padding;
  frame_padding: Padding;
  instrument_spacing: number;
  stave_spacing: number;
  system_start_padding: number;

  instrument_name_font: Font;
  instrument_name_spacing: number;

  systemic_barline_single_instrument_system: boolean;
  bracketing_approach: BracketingApproach;
  bracket_style: BracketStyle;
  bracket_single_staves: boolean;

  base_note_space: number;
  note_space_ratio: number;
  minimum_note_space: number;
  minimum_tie_space: number;

  maximum_beam_slant: number;
};

export type Engraves = {
  by_layout_type: {
    [LayoutType.FullScore]: Engrave;
    [LayoutType.Part]: Engrave;
  };
};

export enum BottomPanelTab {
  Properties,
  Sequencer,
  Mixer,
  Instruments,
}

type ToneSelection = {
  type: "Tone";
  flow_id: UniqueId;
  instrument_id: UniqueId;
  stave_id: UniqueId;
  track_id: UniqueId;
  tone_id: UniqueId;
  tuplet_id?: UniqueId | null;
};

type RestSelection = {
  type: "Rest";
  flow_id: UniqueId;
  instrument_id: UniqueId;
  stave_id: UniqueId;
  track_id: UniqueId;
  tick: number;
  tuplet_id?: UniqueId | null;
};

type TempoSelection = {
  type: "Tempo";
  flow_id: UniqueId;
  tempo_id: UniqueId;
  tick: Tick;
};

type TimeSignatureSelection = {
  type: "TimeSignature";
  flow_id: UniqueId;
  time_signature_id: UniqueId;
  tick: Tick;
};

type KeySignatureSelection = {
  type: "KeySignature";
  flow_id: UniqueId;
  key_signature_id: UniqueId;
  tick: Tick;
};

type ClefSelection = {
  type: "Clef";
  flow_id: UniqueId;
  stave_id: UniqueId;
  clef_id: UniqueId;
  tick: Tick;
};

type PlayingTechniqueSelection = {
  type: "PlayingTechnique";
  flow_id: UniqueId;
  stave_id: UniqueId;
  playing_technique_id: UniqueId;
  tick: Tick;
};

type TupletSelection = {
  type: "Tuplet";
  flow_id: UniqueId;
  instrument_id: UniqueId;
  stave_id: UniqueId;
  track_id: UniqueId;
  tuplet_id: UniqueId;
  tick: Tick;
};

type BarSelection = {
  type: "Bar";
  flow_id: UniqueId;
  tick: Tick;
};

type InsertBarAfterHit = {
  type: "InsertBarAfter";
  flow_id: UniqueId;
  tick: Tick;
};

type DeleteBarHit = {
  type: "DeleteBar";
  flow_id: UniqueId;
  tick: Tick;
};

export type Selection =
  | BarSelection
  | InsertBarAfterHit
  | DeleteBarHit
  | ToneSelection
  | RestSelection
  | TempoSelection
  | TimeSignatureSelection
  | KeySignatureSelection
  | ClefSelection
  | PlayingTechniqueSelection
  | TupletSelection;

export type AppState = {
  single_instrument_transport_ready?: boolean;
  is_project_loading: boolean;
  is_window_focused: boolean;
  ui: {
    tab: Tab;
    layout_id: UniqueId | null;
    flow_id: UniqueId | null;
    renderer: {
      zoom: number;
      pxPerMm: number;
      pan_x: number;
      pan_y: number;
      bounds: { x: number; y: number; width: number; height: number };
      debug: boolean;
    };
    sequencer: {
      x: number;
      y: number;
      tool: Tool;
      snap: NoteDuration;
      instrument: UniqueId | null;
      track: UniqueId | null;
      selection: Selection | null;
    };
    screens: {
      setup: {
        selection: { id: UniqueId; selection_type: SetupSelectionType } | null;
        expanded: UniqueId[];
      };
      write: {
        bottom_panel_minimized: boolean;
        bottom_panel_tab: BottomPanelTab;
      };
      sequence: {
        bottom_panel_minimized: boolean;
        bottom_panel_tab: BottomPanelTab;
        show_active_only: boolean;
      };
    };
  };
  score: {
    players: Players;
    config: Config;
    meta: Meta;
    playback?: ScorePlayback;
    instruments: Instruments;
    flows: Flows;
    tracks: Tracks;
    layouts: Layouts;
    engraves: Engraves;
  };
  resolved_playback_route_by_instrument_id: Record<
    UniqueId,
    ResolvedPlaybackRoute
  >;
  resolved_expression_map_by_instrument_id: Record<
    UniqueId,
    ResolvedExpressionMapLink
  >;
  route_target_options_by_slot_id: Record<UniqueId, PlaybackRouteChoice[]>;
  preferences: {
    audition: boolean;
  };
  current_file_path: string | null;
};

export type DeviceInfo = {
  id: string;
  name: string;
  is_default: boolean;
  sample_rates: number[];
  default_sample_rate: number;
};

export type AudioState = {
  devices: DeviceInfo[];
  active_device_id: string;
  current_sample_rate: number;
};

export type PluginInventoryAvailability = "available" | "unavailable";

export type PluginInventoryFailureKind =
  | "invalid_bundle"
  | "missing_bundle_binary"
  | "unsupported_editor_host"
  | "io_error"
  | "storage_error";

export type PluginInventoryFailure = {
  kind: PluginInventoryFailureKind;
  message: string;
};

export type PluginEditorHostValidationStatus =
  "blocked" | "failed" | "attached";

export type PluginEditorHostValidationPhase =
  | "policy_gate"
  | "module_load"
  | "factory_create"
  | "host_context"
  | "component_create"
  | "component_initialize"
  | "component_runtime_setup"
  | "controller_create"
  | "controller_set_component_handler_pre_init"
  | "controller_initialize"
  | "component_state_sync"
  | "controller_set_component_handler"
  | "connection_point_handshake"
  | "view_create"
  | "view_set_frame"
  | "view_attach"
  | "attached";

export type PluginEditorHostValidationPhaseResultStatus =
  "completed" | "failed" | "skipped";

export type PluginEditorHostValidationPhaseResult = {
  phase: PluginEditorHostValidationPhase;
  status: PluginEditorHostValidationPhaseResultStatus;
  message: string;
};

export type PluginEditorHostLoadInitProbeStatus =
  "blocked_by_host_contract" | "blocked_by_plugin_failure" | "load_init_safe";

export type PluginEditorHostLoadInitProbe = {
  status: PluginEditorHostLoadInitProbeStatus;
  last_successful_phase: PluginEditorHostValidationPhase | null;
  first_failing_phase: PluginEditorHostValidationPhase | null;
  first_failing_message: string | null;
  summary: string;
  phase_results: PluginEditorHostValidationPhaseResult[];
};

export type PluginCapabilityEvidenceStatus = "not_proven";

export type PluginCapabilityEvidence = {
  status: PluginCapabilityEvidenceStatus;
  message: string;
};

export type PluginEditorHostValidation = {
  status: PluginEditorHostValidationStatus;
  phase: PluginEditorHostValidationPhase;
  message: string;
  capability_evidence: PluginCapabilityEvidence | null;
  maintainer_load_init_probe?: PluginEditorHostLoadInitProbe;
};

export type PluginInventoryEntry = {
  id: string;
  name: string;
  bundle_path: string;
  binary_path: string | null;
  source_root: string;
  availability: PluginInventoryAvailability;
  validation_failure: PluginInventoryFailure | null;
  editor_host_validation: PluginEditorHostValidation | null;
};

export type PluginInventoryState = {
  entries: PluginInventoryEntry[];
  errors: PluginInventoryFailure[];
  last_discovered_at_ms: number | null;
};

export type TransportState = {
  playing: boolean;
  current_tick: number;
  tempo: number;
};
