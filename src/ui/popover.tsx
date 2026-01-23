/* eslint-disable jsx-a11y/no-autofocus */

import { ChangeEvent, FC, useEffect, useRef, useState } from "react";
import { actions } from "../services/actions";
import { useAppState } from "../services/state";
import { parseKeySignaturePopover } from "../services/popovers";

export type PopoverState = { x: number; y: number; type: "K" } | null;

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

      if (e.shiftKey && e.key === "K") {
        e.preventDefault();

        const coords = await actions.ui.renderer.getSelectionCoordinates();
        if (coords) {
          const [x, y] = coords;
          setPopover({ x, y, type: "K" });
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

    if (e.key === "Enter") {
      if (value.trim()) {
        const selection = state.ui.sequencer.selection;
        if (selection) {
          let tick: number;
          if (selection.type === "Tone") {
            const tone =
              state.score.tracks.by_id[selection.track_id].entries.by_id[
                selection.tone_id
              ];
            tick = tone.tick;
          } else {
            tick = selection.tick;
          }

          try {
            if (popover.type === "K") {
              parseKeySignaturePopover(value.trim(), selection.flow_id, tick);
            }

            setPopover(null);
            setValue("");
          } catch (error) {
            console.error("Invalid key signature:", error);
          }
        }
      }
    } else if (e.key === "Escape") {
      setPopover(null);
      setValue("");
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

  return (
    <div
      ref={popoverRef}
      className="absolute z-50"
      style={{ left: popover.x, top: popover.y }}
    >
      <div className="shadow-md absolute bottom-0 text-[9px] bg-sky-500 rounded">
        <input
          autoFocus
          autoCorrect="off"
          type="text"
          value={value}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          className="px-2 py-1 text-sm bg-white rounded focus:outline-none border-sky-500 border-2 border-b-0 w-35"
        />
        <div className="px-1 py-px text-white text-center">Key Signature</div>
      </div>
    </div>
  );
};
