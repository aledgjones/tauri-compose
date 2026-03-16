import { FC } from "react";
import classNames from "classnames";
import { Tuplet } from "../../../services/types";
import { useAppState } from "../../../services/state";
import { getTupletHostTicks, hexToRgba } from "./sequencer-tuplet-utils";

type Props = {
  tuplet: Tuplet;
  tickWidth: number;
  height: number;
  color: string;
  barTicks: Set<number>;
};

export const SequencerTrackTuplet: FC<Props> = ({
  tuplet,
  tickWidth,
  height,
  color,
  barTicks,
}) => {
  const selection = useAppState().ui.sequencer.selection;
  const width = getTupletHostTicks(tuplet) * tickWidth;
  const slotWidth = width / tuplet.note_count;
  const hostTicks = getTupletHostTicks(tuplet);
  const fillColor = hexToRgba(color, 0.04);
  const hasSelectedTone =
    selection !== null &&
    selection.type === "Tone" &&
    selection.tuplet_id === tuplet.id;
  const selectedState = hasSelectedTone ? "tone" : "none";

  return (
    <div
      className="absolute top-0 left-0 select-none"
      style={{
        left: tuplet.tick * tickWidth,
        width,
        height,
      }}
      data-tuplet-id={tuplet.id}
      data-selected-state={selectedState}
    >
      <div
        className={classNames("absolute inset-0 rounded-md border", {
          "border-neutral-500": hasSelectedTone,
          "border-neutral-700": !hasSelectedTone,
        })}
        style={{ backgroundColor: fillColor }}
      />
      <div
        className={classNames(
          "absolute left-1 right-1 top-1 h-1 rounded-full",
          {
            "bg-neutral-500": hasSelectedTone,
            "bg-neutral-700": !hasSelectedTone,
          },
        )}
      />
      {Array.from({ length: tuplet.note_count + 1 }, (_, index) => {
        const left = slotWidth * index;
        const absoluteTick =
          tuplet.tick + (hostTicks * index) / tuplet.note_count;
        const isBarStart = barTicks.has(absoluteTick);

        return (
          <div
            key={index}
            className={classNames(
              "absolute top-0 bottom-0 border-l pointer-events-none",
              {
                "border-neutral-500": isBarStart,
                "border-neutral-700": !isBarStart,
              },
            )}
            style={{ left }}
            data-tuplet-slot-marker={tuplet.id}
            data-tuplet-slot-index={index}
          />
        );
      })}
    </div>
  );
};
