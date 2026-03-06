import {
  Accidental,
  Articulation,
  BarlineDrawType,
  ClefDrawType,
  KeySignatureMode,
  NoteDuration,
  PlayerType,
  Tab,
  Tick,
  Ticks,
  TimeSignatureDrawType,
} from "../../../services/types";

export type ScoreFixtureTimeSignature = {
  tick: Tick;
  beats: number;
  beatType: NoteDuration;
  drawType?: TimeSignatureDrawType;
};

export type ScoreFixtureBarline = {
  tick: Tick;
  drawType: BarlineDrawType;
};

export type ScoreFixtureKeySignature = {
  tick: Tick;
  mode: KeySignatureMode;
  offset: number;
};

export type ScoreFixtureClef = {
  tick: Tick;
  drawType: ClefDrawType;
  pitchInt: number;
  offset: number;
  accidental?: Accidental | null;
};

export type ScoreFixtureTone = {
  tick: Tick;
  duration: Ticks;
  pitchInt: number;
  velocity?: number;
  accidental?: Accidental | null;
  articulations?: Articulation[];
};

export type ScoreFixturePayload = {
  title: string;
  activeTab?: Tab;
  instruments: ScoreFixtureInstrument[];
  flows: ScoreFixtureFlow[];
};

export type ScoreFixtureInstrument = {
  id: string;
  definitionId: string;
  playerType?: PlayerType;
};

export type ScoreFixtureFlow = {
  title: string;
  flowLength: Ticks;
  timeSignatures: ScoreFixtureTimeSignature[];
  barlines?: ScoreFixtureBarline[];
  keySignatures?: ScoreFixtureKeySignature[];
  parts: ScoreFixtureFlowPart[];
};

export type ScoreFixtureFlowPart = {
  instrumentId: string;
  staveIndex?: number;
  clefs?: ScoreFixtureClef[];
  tones: ScoreFixtureTone[];
};

export type ScoreFixtureCase = {
  flowTitle: string;
  name: string;
  measure: number;
  timeSignature: string;
};

export type ScoreFixtureDefinition = {
  id: string;
  name: string;
  description: string;
  payload: ScoreFixturePayload;
  cases: ScoreFixtureCase[];
};
