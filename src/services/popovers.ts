import { actions } from "./actions";
import {
  BarlineDrawType,
  ClefDrawType,
  KeySignatureMode,
  NoteDuration,
  Tick,
  TimeSignatureDrawType,
  UniqueId,
} from "./types";
import { parseTimeSignature } from "./utils";

export type PopoverType =
  | "barline"
  | "clef"
  | "dynamic"
  | "measure"
  | "key-signature";

export const parseBarlinePopover = (
  input: string,
  flow_id: UniqueId,
  tick: Tick
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
        BarlineDrawType.EndStartRepeat
      );
      break;
    }

    default: {
      throw `Unknown input: ${input}`;
    }
  }
};

export const parseClefPopover = (
  input: string,
  flow_id: UniqueId,
  tick: Tick,
  stave_id?: UniqueId
) => {
  if (!stave_id) {
    return;
  }

  switch (input) {
    case "clear": {
      actions.score.clef.clear(flow_id, tick, stave_id);
      break;
    }

    case "treble":
    case "g":
    case "G":
    case "sol": {
      actions.score.clef.create(flow_id, tick, stave_id, 67, 2, ClefDrawType.G);
      break;
    }

    case "bass":
    case "f":
    case "F":
    case "fa": {
      actions.score.clef.create(
        flow_id,
        tick,
        stave_id,
        53,
        -2,
        ClefDrawType.F
      );
      break;
    }

    case "alto":
    case "ca":
    case "CA": {
      actions.score.clef.create(flow_id, tick, stave_id, 60, 0, ClefDrawType.C);
      break;
    }

    case "tenor":
    case "ct":
    case "CT": {
      actions.score.clef.create(
        flow_id,
        tick,
        stave_id,
        60,
        -2,
        ClefDrawType.C
      );
      break;
    }

    default: {
      throw `Unknown input: ${input}`;
    }
  }
};

export const parseKeySignaturePopover = (
  input: string,
  flow_id: UniqueId,
  tick: Tick
) => {
  switch (input) {
    case "clear": {
      actions.score.keySignature.clearAtTick(flow_id, tick);
      break;
    }

    case "C#":
    case "7s":
    case "7#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        7
      );
      break;
    }

    case "F#":
    case "6s":
    case "6#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        6
      );
      break;
    }

    case "B":
    case "5s":
    case "5#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        5
      );
      break;
    }

    case "E":
    case "4s":
    case "4#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        4
      );
      break;
    }

    case "A":
    case "3s":
    case "3#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        3
      );
      break;
    }

    case "D":
    case "2s":
    case "2#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        2
      );
      break;
    }

    case "G":
    case "1s":
    case "1#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        1
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
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        0
      );
      break;
    }

    case "F":
    case "1f":
    case "1b": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -1
      );
      break;
    }

    case "Bb":
    case "2f":
    case "2b": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -2
      );
      break;
    }

    case "Eb":
    case "3f":
    case "3b": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -3
      );
      break;
    }

    case "Ab":
    case "4f":
    case "4b": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -4
      );
      break;
    }

    case "Db":
    case "5f":
    case "5b": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -5
      );
      break;
    }

    case "Gb":
    case "6f":
    case "6b": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -6
      );
      break;
    }

    case "Cb":
    case "7f":
    case "7b": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Major,
        -7
      );
      break;
    }

    case "a#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        7
      );
      break;
    }

    case "d#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        6
      );
      break;
    }

    case "g#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        5
      );
      break;
    }

    case "c#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        4
      );
      break;
    }

    case "f#": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        3
      );
      break;
    }

    case "b": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        2
      );
      break;
    }

    case "e": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        1
      );
      break;
    }

    case "a": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        0
      );
      break;
    }

    case "d": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -1
      );
      break;
    }

    case "g": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -2
      );
      break;
    }

    case "c": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -3
      );
      break;
    }

    case "f": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -4
      );
      break;
    }

    case "bb": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -5
      );
      break;
    }

    case "eb": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -6
      );
      break;
    }

    case "ab": {
      actions.score.keySignature.create(
        flow_id,
        tick,
        KeySignatureMode.Minor,
        -7
      );
      break;
    }

    default: {
      throw `Unknown input: ${input}`;
    }
  }
};

export const parseTimeSignaturePopover = (
  input: string,
  flow_id: UniqueId,
  tick: Tick
) => {
  const parsed = parseTimeSignature(input);

  if (parsed) {
    switch (parsed.beatType) {
      case 1: {
        actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Whole,
          TimeSignatureDrawType.Regular
        );
        return;
      }
      case 2: {
        actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Half,
          TimeSignatureDrawType.Regular
        );
        return;
      }
      case 4: {
        actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Quarter,
          TimeSignatureDrawType.Regular
        );
        return;
      }
      case 8: {
        actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Eighth,
          TimeSignatureDrawType.Regular
        );
        return;
      }
      case 16: {
        actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.Sixteenth,
          TimeSignatureDrawType.Regular
        );
        return;
      }
      case 32: {
        actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.ThirtySecond,
          TimeSignatureDrawType.Regular
        );
        return;
      }
      case 64: {
        actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.SixtyFourth,
          TimeSignatureDrawType.Regular
        );
        return;
      }
      case 128: {
        actions.score.timeSignature.create(
          flow_id,
          tick,
          parsed.beats,
          NoteDuration.HudredTwentyEighth,
          TimeSignatureDrawType.Regular
        );
        return;
      }

      default:
        break;
    }
  }

  switch (input) {
    case "clear": {
      actions.score.barline.clearAtTick(flow_id, tick);
      break;
    }

    case "c": {
      actions.score.timeSignature.create(
        flow_id,
        tick,
        4,
        NoteDuration.Quarter,
        TimeSignatureDrawType.CommonTime
      );
      break;
    }

    case "cutc": {
      actions.score.timeSignature.create(
        flow_id,
        tick,
        2,
        NoteDuration.Half,
        TimeSignatureDrawType.SplitCommonTime
      );
      break;
    }

    default: {
      throw `Unknown input: ${input}`;
    }
  }
};
