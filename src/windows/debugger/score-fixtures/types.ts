import {
  Accidental,
  Articulation,
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
  parts: ScoreFixtureFlowPart[];
};

export type ScoreFixtureFlowPart = {
  instrumentId: string;
  staveIndex?: number;
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
