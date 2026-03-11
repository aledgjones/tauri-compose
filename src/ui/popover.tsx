/* eslint-disable jsx-a11y/no-autofocus */

import { ChangeEvent, FC, useEffect, useRef, useState } from "react";
import { actions } from "../services/actions";
import { useAppState } from "../services/state";
import {
  parseBarlinePopover,
  parseClefPopover,
  parseKeySignaturePopover,
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
  mdiMetronome,
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
    const tone = track?.entries.by_id[selection.tone_id];

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

  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (popover) {
        return;
      }

      if (e.shiftKey && e.key === "B") {
        e.preventDefault();

        const coords = await actions.ui.renderer.getSelectionCoordinates();
        if (coords) {
          const [x, y] = coords;
          setPopover({ x, y, type: PopoverType.Barline });
        }
      }

      if (e.shiftKey && e.key === "C") {
        e.preventDefault();

        const coords = await actions.ui.renderer.getSelectionCoordinates();
        if (coords) {
          const [x, y] = coords;
          setPopover({ x, y, type: PopoverType.Clef });
        }
      }

      if (e.shiftKey && e.key === "K") {
        e.preventDefault();

        const coords = await actions.ui.renderer.getSelectionCoordinates();
        if (coords) {
          const [x, y] = coords;
          setPopover({ x, y, type: PopoverType.KeySignature });
        }
      }

      if (e.shiftKey && e.key === "M") {
        e.preventDefault();

        const coords = await actions.ui.renderer.getSelectionCoordinates();
        if (coords) {
          const [x, y] = coords;
          setPopover({ x, y, type: PopoverType.TimeSignature });
        }
      }

      if (e.shiftKey && e.key === "T") {
        e.preventDefault();

        const coords = await actions.ui.renderer.getSelectionCoordinates();
        if (coords) {
          const [x, y] = coords;
          setPopover({ x, y, type: PopoverType.Tempo });
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [popover, setPopover]);

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

  const handleKeyDown = async (e: React.KeyboardEvent) => {
    e.stopPropagation();

    switch (e.key) {
      case "Enter": {
        const term = value.trim();
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
          }

          setPopover(null);
          setValue("");
        } catch (error) {
          console.error("Invalid input:", error);
        }

        break;
      }
      case "Escape": {
        setPopover(null);
        setValue("");
        break;
      }
    }
  };

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setValue(newValue);
  };

  const handleBlur = () => {
    setPopover(null);
    setValue("");
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
