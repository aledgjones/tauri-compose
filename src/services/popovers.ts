import { actions } from "./actions";
import { BarlineDrawType, KeySignatureMode, Tick, UniqueId } from "./types";

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

    case "|]":
    case "final": {
      actions.score.barline.create(flow_id, tick, BarlineDrawType.Final);
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

    default: {
      throw `Unknown input: ${input}`;
    }
  }
};
