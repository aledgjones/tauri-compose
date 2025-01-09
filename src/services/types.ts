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
  id: number;
  definition_id: string;
  instrument_type: InstrumentType;
  long_name: string;
  short_name: string;
  staves: number[];
  count: string | null;
  volume: number;
  solo: boolean;
  mute: boolean;
};

export type Instruments = {
  by_id: Record<number, Instrument>;
};

export type Player = {
  id: number;
  player_type: PlayerType;
  instruments: number[];
};

export type Players = {
  order: number[];
  by_id: Record<number, Player>;
};

export type AppState = {
  ui: {
    tab: Tab;
    layout_id: string;
    setup: {
      selection: { id: number; selection_type: SelectionType } | null;
      expanded: number[];
    };
  };
  score: {
    players: Players;
    config: Config;
    meta: Meta;
    instruments: Instruments;
  };
};
