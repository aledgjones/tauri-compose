import { actions } from "./actions";
import { BarlineDrawType, Tick, UniqueId } from "./types";

export type PopoverType = "barline" | "clef" | "dynamic" | "measure";

export const parseBarlinePopover = (
  input: string,
  flow_id: UniqueId,
  tick: Tick
) => {
  switch (input) {
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

    default: {
      throw `Unknown input: ${input}`;
    }
  }
};
