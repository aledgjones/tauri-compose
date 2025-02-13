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
  Hidden = 0,
  G,
  F,
  C,
  Percussion,
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
  None,
  Staccato,
  Staccatissimo,
  Tenuto,
  StaccatoTenuto,
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

export enum SelectionType {
  Player,
  Flow,
  Layout,
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
  volume: number;
  solo: boolean;
  mute: boolean;
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

export type TimeSignature = {
  id: UniqueId;
  tick: Tick;
  type: "TimeSignature";
  beats: number;
  beat_type: NoteDuration;
};

export type KeySignature = {
  id: UniqueId;
  tick: Tick;
  type: "KeySignature";
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
  articulation: Articulation;
};

export type Entry = TimeSignature | KeySignature | Tone;

export type Track = {
  id: UniqueId;
  entries: {
    by_tick: Record<UniqueId, UniqueId[]>;
    by_id: Record<UniqueId, Entry>;
  };
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
  subdivisions: number;
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

export type AppState = {
  ui: {
    tab: Tab;
    layout_id: UniqueId;
    setup: {
      selection: { id: UniqueId; selection_type: SelectionType } | null;
      expanded: UniqueId[];
    };
    sequencer: {
      x: number;
      y: number;
      tool: Tool;
      snap: NoteDuration;
      flow: UniqueId | null;
      instrument: UniqueId | null;
      track: UniqueId | null;
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
};
