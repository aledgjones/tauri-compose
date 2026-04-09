/* eslint-disable jsx-a11y/no-autofocus */

import {
  ChangeEvent,
  FC,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import type { KeyboardEvent as ReactKeyboardEvent } from "react";
import { actions } from "../services/actions";
import { useAppState } from "../services/state";
import {
  parseTupletPopover,
  parseBarlinePopover,
  parseClefPopover,
  parseKeySignaturePopover,
  parsePlayingTechniquePopover,
  parseTempoPopover,
  parseTimeSignaturePopover,
  PopoverType,
} from "../services/popovers";
import { Icon } from "./icon";
import {
  mdiAlphaBBoxOutline,
  mdiAlphaCBoxOutline,
  mdiAlphaKBoxOutline,
  mdiAlphaMBoxOutline,
  mdiAlphaPBoxOutline,
  mdiMetronome,
  mdiNumeric3BoxOutline,
} from "@mdi/js";
import { AppState } from "../services/types";

export type PopoverState = { x: number; y: number; type: PopoverType } | null;

const getTickFromSelection = (state: AppState) => {
  const selection = state.ui.sequencer.selection;
  if (!selection) {
    return null;
  }

  if (selection.type === "Tone") {
    const track = state.score.tracks.by_id[selection.track_id];
    if (!track) {
      return null;
    }

    if (selection.tuplet_id !== undefined && selection.tuplet_id !== null) {
      const tuplet = track.entries.by_id[selection.tuplet_id];

      if (!tuplet || tuplet.type !== "Tuplet") {
        return null;
      }

      const tone = tuplet.entries.by_id[selection.tone_id];

      if (!tone || tone.type !== "Tone") {
        return null;
      }

      return tone.tick;
    }

    const tone = track.entries.by_id[selection.tone_id];

    if (!tone || tone.type !== "Tone") {
      return null;
    }

    return tone.tick;
  } else {
    return selection.tick;
  }
};

const getFlowIdFromSelection = (state: AppState) => {
  const selection = state.ui.sequencer.selection;

  if (!selection) {
    return null;
  }

  return selection.flow_id;
};

const getStaveIdFromSelection = (state: AppState) => {
  const selection = state.ui.sequencer.selection;

  if (
    !selection ||
    selection.type === "Tempo" ||
    selection.type === "TimeSignature" ||
    selection.type === "KeySignature"
  ) {
    return null;
  }

  return selection.stave_id;
};

const getFirstStaveIdForFlow = (state: AppState, flowId: number) => {
  const flow = state.score.flows.by_id[flowId];

  if (!flow) {
    return null;
  }

  for (const playerId of state.score.players.order) {
    if (!flow.player_ids.includes(playerId)) {
      continue;
    }

    const player = state.score.players.by_id[playerId];

    if (!player) {
      continue;
    }

    for (const instrumentId of player.instrument_ids) {
      const instrument = state.score.instruments.by_id[instrumentId];

      if (!instrument) {
        continue;
      }

      for (const staveId of instrument.stave_ids) {
        if (flow.staves.by_id[staveId]) {
          return staveId;
        }
      }
    }
  }

  return null;
};

const getTrackIdFromSelection = (state: AppState) => {
  const selection = state.ui.sequencer.selection;

  if (
    !selection ||
    selection.type === "Tempo" ||
    selection.type === "TimeSignature" ||
    selection.type === "KeySignature" ||
    selection.type === "Clef" ||
    selection.type === "PlayingTechnique"
  ) {
    return null;
  }

  return selection.track_id;
};

const getInstrumentIdFromSelection = (state: AppState) => {
  const selection = state.ui.sequencer.selection;

  if (
    !selection ||
    selection.type === "Tempo" ||
    selection.type === "TimeSignature" ||
    selection.type === "KeySignature" ||
    selection.type === "Clef" ||
    selection.type === "PlayingTechnique"
  ) {
    return null;
  }

  return selection.instrument_id;
};

const canOpenTupletPopover = (state: AppState) => {
  const selection = state.ui.sequencer.selection;

  return selection?.type === "Tone" || selection?.type === "Rest";
};

const isEditableTarget = (target: EventTarget | null) => {
  if (!(target instanceof HTMLElement)) {
    return false;
  }

  if (target.isContentEditable || target.closest('[contenteditable="true"]')) {
    return true;
  }

  if (target instanceof HTMLTextAreaElement) {
    return true;
  }

  if (target instanceof HTMLInputElement) {
    return target.type !== "button";
  }

  return (
    target.getAttribute("role") === "textbox" ||
    target.closest('[role="textbox"]') !== null
  );
};

const getMeta = (type: PopoverType) => {
  switch (type) {
    case PopoverType.Barline: {
      return {
        label: "Barline",
        icon: mdiAlphaBBoxOutline,
      };
    }
    case PopoverType.Clef: {
      return {
        label: "Clef",
        icon: mdiAlphaCBoxOutline,
      };
    }
    case PopoverType.KeySignature: {
      return {
        label: "Key Signature",
        icon: mdiAlphaKBoxOutline,
      };
    }
    case PopoverType.TimeSignature: {
      return {
        label: "Time Signature",
        icon: mdiAlphaMBoxOutline,
      };
    }
    case PopoverType.Tempo: {
      return {
        label: "Tempo",
        icon: mdiMetronome,
      };
    }
    case PopoverType.Tuplet: {
      return {
        label: "Tuplet",
        icon: mdiNumeric3BoxOutline,
      };
    }
    case PopoverType.PlayingTechnique: {
      return {
        label: "Playing Technique",
        icon: mdiAlphaPBoxOutline,
      };
    }
    default:
      return { label: "", icon: mdiAlphaKBoxOutline };
  }
};

type Props = {
  popover: PopoverState;
  setPopover: (popover: PopoverState) => void;
};

export const Popover: FC<Props> = ({ popover, setPopover }) => {
  const [value, setValue] = useState("");
  const popoverRef = useRef<HTMLDivElement>(null);
  const state = useAppState();

  const closePopover = () => {
    setPopover(null);
    setValue("");
  };

  const openPopoverAtSelection = useCallback(
    async (type: PopoverType) => {
      const coords = await actions.ui.renderer.getSelectionCoordinates();
      if (!coords) {
        return;
      }

      const [x, y] = coords;
      setPopover({ x, y, type });
    },
    [setPopover],
  );

  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (popover) {
        return;
      }

      if (e.defaultPrevented || isEditableTarget(e.target)) {
        return;
      }

      if (e.shiftKey && e.key === "B") {
        e.preventDefault();
        await openPopoverAtSelection(PopoverType.Barline);
      }

      if (e.shiftKey && e.key === "C") {
        e.preventDefault();
        await openPopoverAtSelection(PopoverType.Clef);
      }

      if (e.shiftKey && e.key === "K") {
        e.preventDefault();
        await openPopoverAtSelection(PopoverType.KeySignature);
      }

      if (e.shiftKey && e.key === "M") {
        e.preventDefault();
        await openPopoverAtSelection(PopoverType.TimeSignature);
      }

      if (e.shiftKey && e.key === "T") {
        e.preventDefault();
        await openPopoverAtSelection(PopoverType.Tempo);
      }

      if (e.shiftKey && e.key === "P") {
        e.preventDefault();
        await openPopoverAtSelection(PopoverType.PlayingTechnique);
      }

      if (
        !e.shiftKey &&
        !e.metaKey &&
        !e.ctrlKey &&
        !e.altKey &&
        canOpenTupletPopover(state) &&
        e.key === ";"
      ) {
        e.preventDefault();
        await openPopoverAtSelection(PopoverType.Tuplet);
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [openPopoverAtSelection, popover, setPopover, state]);

  // stop the popover from closing when clicking inside it
  useEffect(() => {
    if (!popover || !popoverRef.current) return;

    const handlePointerDown = (e: PointerEvent) => {
      e.preventDefault();
      e.stopPropagation();
    };

    const $element = popoverRef.current;
    $element.addEventListener("pointerdown", handlePointerDown);

    return () => $element.removeEventListener("pointerdown", handlePointerDown);
  }, [popover]);

  if (!popover) {
    return null;
  }

  const handleKeyDown = async (e: ReactKeyboardEvent<HTMLInputElement>) => {
    e.stopPropagation();

    switch (e.key) {
      case "Enter": {
        const term = value?.trim() ?? "";
        const tick = getTickFromSelection(state);
        const flow_id = getFlowIdFromSelection(state);

        if (!term || flow_id === null || tick === null) {
          return;
        }

        try {
          switch (popover.type) {
            case PopoverType.Barline: {
              parseBarlinePopover(term, flow_id, tick);
              break;
            }
            case PopoverType.Clef: {
              const stave_id = getStaveIdFromSelection(state);

              if (!stave_id) {
                return;
              }

              await parseClefPopover(term, flow_id, tick, stave_id);
              break;
            }
            case PopoverType.KeySignature: {
              await parseKeySignaturePopover(term, flow_id, tick);
              break;
            }
            case PopoverType.TimeSignature: {
              await parseTimeSignaturePopover(term, flow_id, tick);
              break;
            }
            case PopoverType.Tempo: {
              await parseTempoPopover(term, flow_id, tick);
              break;
            }
            case PopoverType.PlayingTechnique: {
              const stave_id =
                getStaveIdFromSelection(state) ??
                getFirstStaveIdForFlow(state, flow_id);

              if (!stave_id) {
                return;
              }

              await parsePlayingTechniquePopover(term, flow_id, tick, stave_id);
              break;
            }
            case PopoverType.Tuplet: {
              if (!canOpenTupletPopover(state)) {
                return;
              }

              const instrument_id = getInstrumentIdFromSelection(state);
              const stave_id = getStaveIdFromSelection(state);
              const track_id = getTrackIdFromSelection(state);

              if (!stave_id || !track_id || !instrument_id) {
                return;
              }

              await parseTupletPopover(
                term,
                flow_id,
                tick,
                instrument_id,
                stave_id,
                track_id,
              );
              break;
            }
          }

          closePopover();
        } catch (error) {
          console.error("Invalid input:", error);
        }

        break;
      }
      case "Escape": {
        closePopover();
        break;
      }
    }
  };

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setValue(newValue);
  };

  const handleBlur = () => {
    closePopover();
  };

  const meta = getMeta(popover.type);

  return (
    <div
      ref={popoverRef}
      className="absolute z-50"
      style={{ left: popover.x, top: popover.y }}
    >
      <div className="shadow-lg absolute bottom-0 text-[9px] bg-sky-500 rounded">
        <input
          autoFocus
          autoCorrect="off"
          type="text"
          value={value}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          className="px-2 py-1 text-sm bg-white rounded-xs focus:outline-none m-0.5 mb-0 border-b-0 w-40"
        />
        <div className="flex items-center px-1 py-px text-white">
          <Icon className="-ml-0.5 mr-0.5" size={0.5} path={meta.icon} />
          <span>{meta.label}</span>
        </div>
      </div>
    </div>
  );
};
