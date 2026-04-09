import { actions } from "./actions";
import {
  BarlineDrawType,
  ClefDrawType,
  KeySignatureMode,
  NoteDuration,
  PlayingTechniqueKind,
  Tick,
  TimeSignatureDrawType,
  UniqueId,
} from "./types";
import { parseTempo, parseTimeSignature } from "./utils";

export enum PopoverType {
  Barline = 0,
  Clef,
  KeySignature,
  TimeSignature,
  Tempo,
  Tuplet,
  PlayingTechnique,
}

const TUPLET_BASE_NOTE_BY_SUFFIX: Record<string, NoteDuration> = {
  q: NoteDuration.Quarter,
  e: NoteDuration.Eighth,
  s: NoteDuration.Sixteenth,
  t: NoteDuration.ThirtySecond,
};

export const parseTupletPopover = async (
  input: string,
  flow_id: UniqueId,
  tick: Tick,
  instrument_id: UniqueId,
  stave_id: UniqueId,
  track_id: UniqueId,
) => {
  const match = input.match(/^(\d+)([qest])\/(\d+)$/i);

  if (!match) {
    throw new Error(`Unknown input: ${input}`);
  }

  const [, noteCountText, durationSuffix, inSpaceOfText] = match;
  const note_count = Number(noteCountText);
  const in_space_of = Number(inSpaceOfText);
  const base_note_value =
    TUPLET_BASE_NOTE_BY_SUFFIX[durationSuffix.toLowerCase()];

  if (note_count !== 3 || in_space_of !== 2 || base_note_value === undefined) {
    throw new Error(`Unknown input: ${input}`);
  }

  const tuplet_id = await actions.score.tuplet.create(
    track_id,
    tick,
    note_count,
    in_space_of,
    base_note_value,
    [3],
  );

  await actions.ui.sequencer.selectTuplet(
    flow_id,
    instrument_id,
    stave_id,
    track_id,
    tuplet_id,
    tick,
  );
};

export const parseBarlinePopover = (
  input: string,
  flow_id: UniqueId,
  tick: Tick,
) => {
  switch (input) {
    case "clear": {
      actions.score.barline.clearAtTick(flow_id, tick);
      break;
    }

    case "|":
    case "single":
    case "normal": {
      actions.score.barline.create(flow_id, tick, BarlineDrawType.Single);
      break;
    }

    case "||":
    case "double": {
      actions.score.barline.create(flow_id, tick, BarlineDrawType.Double);
      break;
    }

    case "|||":
    case "triple": {
      actions.score.barline.create(flow_id, tick, BarlineDrawType.Triple);
      break;
    }

    case ":":
    case "dashed": {
      actions.score.barline.create(flow_id, tick, BarlineDrawType.Dashed);
      break;
    }

    case "|:":
    case "start": {
      actions.score.barline.create(flow_id, tick, BarlineDrawType.StartRepeat);
      break;
    }

    case ":|":
    case "end": {
      actions.score.barline.create(flow_id, tick, BarlineDrawType.EndRepeat);
      break;
    }

    case ":|:":
    case ":||:":
    case "end-start":
    case "endstart": {
      actions.score.barline.create(
        flow_id,
        tick,
        BarlineDrawType.EndStartRepeat,
      );
      break;
    }

    default: {
      throw new Error(`Unknown input: ${input}`);
    }
  }
};

export const parseClefPopover = async (
  input: string,
  flow_id: UniqueId,
  tick: Tick,
  stave_id: UniqueId,
) => {
  switch (input) {
    case "treble":
    case "g":
    case "G":
    case "sol": {
      const clef_id = await actions.score.clef.create(
        flow_id,
        tick,
        stave_id,
        67,
        2,
        ClefDrawType.G,
      );
      if (clef_id !== null) {
        await actions.ui.sequencer.selectClef(flow_id, stave_id, clef_id, tick);
      }
      break;
    }

    case "bass":
    case "f":
    case "F":
    case "fa": {
      const clef_id = await actions.score.clef.create(
        flow_id,
        tick,
        stave_id,
        53,
        -2,
        ClefDrawType.F,
      );
      if (clef_id !== null) {
        await actions.ui.sequencer.selectClef(flow_id, stave_id, clef_id, tick);
      }
      break;
    }

    case "alto":
    case "ca":
    case "CA": {
      const clef_id = await actions.score.clef.create(
        flow_id,
        tick,
        stave_id,
        60,
        0,
        ClefDrawType.C,
      );
      if (clef_id !== null) {
        await actions.ui.sequencer.selectClef(flow_id, stave_id, clef_id, tick);
      }
      break;
    }

    case "tenor":
    case "ct":
    case "CT": {
      const clef_id = await actions.score.clef.create(
        flow_id,
        tick,
        stave_id,
        60,
        -2,
        ClefDrawType.C,
      );
      if (clef_id !== null) {
        await actions.ui.sequencer.selectClef(flow_id, stave_id, clef_id, tick);
      }
      break;
    }

    case "percussion":
    case "perc":
    case "PERC": {
      const clef_id = await actions.score.clef.create(
        flow_id,
        tick,
        stave_id,
        60,
        0,
        ClefDrawType.Percussion,
      );
      if (clef_id !== null) {
        await actions.ui.sequencer.selectClef(flow_id, stave_id, clef_id, tick);
      }
      break;
    }

    default: {
      throw new Error(`Unknown input: ${input}`);
    }
  }
};

export const parseKeySignaturePopover = async (
  input: string,
  flow_id: UniqueId,
  tick: Tick,
) => {
  switch (input) {
    case "C#":
    case "7s":
    case "7#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        7,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "F#":
    case "6s":
    case "6#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        6,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "B":
    case "5s":
    case "5#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        5,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "E":
    case "4s":
    case "4#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        4,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "A":
    case "3s":
    case "3#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        3,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "D":
    case "2s":
    case "2#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        2,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "G":
    case "1s":
    case "1#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        1,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "open":
    case "atonal":
    case "C":
    case "0s":
    case "0f":
    case "0#":
    case "0b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        0,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "F":
    case "1f":
    case "1b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -1,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "Bb":
    case "2f":
    case "2b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -2,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "Eb":
    case "3f":
    case "3b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -3,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "Ab":
    case "4f":
    case "4b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -4,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "Db":
    case "5f":
    case "5b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -5,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "Gb":
    case "6f":
    case "6b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -6,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "Cb":
    case "7f":
    case "7b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -7,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "a#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        7,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "d#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        6,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "g#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        5,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "c#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        4,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "f#": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        3,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "b": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        2,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "e": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        1,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "a": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        0,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "d": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -1,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "g": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -2,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "c": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -3,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "f": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -4,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "bb": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -5,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "eb": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -6,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    case "ab": {
      const key_signature_id = await actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -7,
      );
      await actions.ui.sequencer.selectKeySignature(
        flow_id,
        key_signature_id,
        tick,
      );
      break;
    }

    default: {
      throw new Error(`Unknown input: ${input}`);
    }
  }
};

export const parseTimeSignaturePopover = async (
  input: string,
  flow_id: UniqueId,
  tick: Tick,
) => {
  const parsed = parseTimeSignature(input);

  if (parsed) {
    switch (parsed.beatType) {
      case 1: {
        const time_signature_id = await actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Whole,
          TimeSignatureDrawType.Regular,
          false,
        );
        await actions.ui.sequencer.selectTimeSignature(
          flow_id,
          time_signature_id,
          tick,
        );
        return;
      }
      case 2: {
        const time_signature_id = await actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Half,
          TimeSignatureDrawType.Regular,
          false,
        );
        await actions.ui.sequencer.selectTimeSignature(
          flow_id,
          time_signature_id,
          tick,
        );
        return;
      }
      case 4: {
        const time_signature_id = await actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Quarter,
          TimeSignatureDrawType.Regular,
          false,
        );
        await actions.ui.sequencer.selectTimeSignature(
          flow_id,
          time_signature_id,
          tick,
        );
        return;
      }
      case 8: {
        const time_signature_id = await actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Eighth,
          TimeSignatureDrawType.Regular,
          false,
        );
        await actions.ui.sequencer.selectTimeSignature(
          flow_id,
          time_signature_id,
          tick,
        );
        return;
      }
      case 16: {
        const time_signature_id = await actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Sixteenth,
          TimeSignatureDrawType.Regular,
          false,
        );
        await actions.ui.sequencer.selectTimeSignature(
          flow_id,
          time_signature_id,
          tick,
        );
        return;
      }
      case 32: {
        const time_signature_id = await actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.ThirtySecond,
          TimeSignatureDrawType.Regular,
          false,
        );
        await actions.ui.sequencer.selectTimeSignature(
          flow_id,
          time_signature_id,
          tick,
        );
        return;
      }
      case 64: {
        const time_signature_id = await actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.SixtyFourth,
          TimeSignatureDrawType.Regular,
          false,
        );
        await actions.ui.sequencer.selectTimeSignature(
          flow_id,
          time_signature_id,
          tick,
        );
        return;
      }
      case 128: {
        const time_signature_id = await actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.HudredTwentyEighth,
          TimeSignatureDrawType.Regular,
          false,
        );
        await actions.ui.sequencer.selectTimeSignature(
          flow_id,
          time_signature_id,
          tick,
        );
        return;
      }

      default:
        break;
    }
  }

  switch (input) {
    case "c": {
      const time_signature_id = await actions.score.timeSignature.create(
        flow_id,
        tick,
        4,
        NoteDuration.Quarter,
        TimeSignatureDrawType.CommonTime,
        false,
      );
      await actions.ui.sequencer.selectTimeSignature(
        flow_id,
        time_signature_id,
        tick,
      );
      break;
    }

    case "cutc": {
      const time_signature_id = await actions.score.timeSignature.create(
        flow_id,
        tick,
        2,
        NoteDuration.Half,
        TimeSignatureDrawType.SplitCommonTime,
        false,
      );
      await actions.ui.sequencer.selectTimeSignature(
        flow_id,
        time_signature_id,
        tick,
      );
      break;
    }

    default: {
      throw new Error(`Unknown input: ${input}`);
    }
  }
};

export const parseTempoPopover = async (
  input: string,
  flow_id: UniqueId,
  tick: Tick,
) => {
  const parsed = parseTempo(input);

  if (!parsed) {
    throw new Error(`Unknown input: ${input}`);
  }

  const tempo_id = await actions.score.tempo.create(
    flow_id,
    tick,
    parsed.text,
    parsed.beat_type,
    parsed.dots,
    parsed.bpm,
    parsed.hidden,
    parsed.show_text,
    parsed.show_metronome,
    parsed.parenthesized,
  );

  await actions.ui.sequencer.selectTempo(tempo_id, flow_id, tick);
};

export const parsePlayingTechniquePopover = async (
  input: string,
  flow_id: UniqueId,
  tick: Tick,
  stave_id: UniqueId,
) => {
  let kind: PlayingTechniqueKind;

  switch (input) {
    case "pizz": {
      kind = PlayingTechniqueKind.Pizz;
      break;
    }
    case "arco": {
      kind = PlayingTechniqueKind.Arco;
      break;
    }
    default: {
      throw new Error(`Unknown input: ${input}`);
    }
  }

  const playing_technique_id = await actions.score.playingTechnique.create(
    flow_id,
    tick,
    stave_id,
    kind,
    false,
  );

  await actions.ui.sequencer.selectPlayingTechnique(
    flow_id,
    stave_id,
    playing_technique_id,
    tick,
  );
};
