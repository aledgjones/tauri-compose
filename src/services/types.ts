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
  };
};
