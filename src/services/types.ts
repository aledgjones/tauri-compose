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

export type Instrument = {
  id: UniqueId;
  definition_id: string;
  instrument_type: InstrumentType;
  long_name: string;
  short_name: string;
  stave_ids: UniqueId[];
  count: string | null;
  assigned_plugin_id: string | null;
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

type TupletSelection = {
  type: "Tuplet";
  flow_id: UniqueId;
  instrument_id: UniqueId;
  stave_id: UniqueId;
  track_id: UniqueId;
  tuplet_id: UniqueId;
  tick: Tick;
};

export type Selection =
  | ToneSelection
  | RestSelection
  | TempoSelection
  | TimeSignatureSelection
  | KeySignatureSelection
  | ClefSelection
  | TupletSelection;

export type AppState = {
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
    instruments: Instruments;
    flows: Flows;
    tracks: Tracks;
    layouts: Layouts;
    engraves: Engraves;
  };
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

export type ArticulationInfo = {
  id: string;
  name: string;
  range: Array<[number, number]>;
};

export type PluginInfo = {
  id: string;
  name: string;
  path: string;
  author: string;
  version: string;
  cover_image?: string;
  articulations: ArticulationInfo[];
};

export type SamplerState = {
  plugins: PluginInfo[];
  loaded_plugin_ids: string[];
};

export type TransportState = {
  playing: boolean;
  current_tick: number;
  tempo: number;
};
