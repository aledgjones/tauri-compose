export type UniqueId = number;

export type Log = {
  type: "log" | "error" | "warn";
  message: string;
};

export enum Tab {
  Setup = 0,
  Write,
  Engrave,
  Sequence,
  Print,
}

export enum SelectionType {
  Player = 0,
  Flow,
  Layout,
}

export enum PlayerType {
  Solo = 0,
  Section,
}

export enum AutoCountStyle {
  Arabic = 0,
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

export type Track = {
  id: UniqueId;
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
  staves: Staves;
  master_id: UniqueId;
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
    layout_id: string;
    setup: {
      selection: { id: UniqueId; selection_type: SelectionType } | null;
      expanded: UniqueId[];
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
