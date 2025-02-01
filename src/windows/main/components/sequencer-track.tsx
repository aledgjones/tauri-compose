import { FC, PointerEvent, useCallback, useRef } from "react";
import { TickConfig } from "../hooks/use-ticks";
import { KeyType, Slots } from "../hooks/use-slots";
import classNames from "classnames";
import {
  Articulation,
  Flow,
  NoteDuration,
  Tool,
  Track,
} from "../../../services/types";
import { actions } from "../../../services/actions";

export const getSnappedXPosition = (
  x: number,
  tickWidth: number,
  ticksPerSnap: number,
  direction: "up" | "down" | "nearest"
) => {
  const tick = Math.floor(x / tickWidth);

  const extra = tick % ticksPerSnap;

  if (extra === 0) {
    return tick;
  }

  switch (direction) {
    case "up":
      return tick - extra + ticksPerSnap;
    case "down":
      return tick - extra;
    default:
      return extra < ticksPerSnap / 2
        ? tick - extra
        : tick - extra + ticksPerSnap;
  }
};

export const getPitchFromYPosition = (y: number, slotHeight: number) => {
  const slot = Math.floor(y / slotHeight);
  const lowestNote = 12;
  const highestNote = lowestNote + 10 * 12 - 1;
  let int = highestNote - slot;

  if (int < 12) {
    int = 12;
  }

  if (int > 131) {
    int = 131;
  }

  return int;
};

type SequencerTrackProps = {
  flow: Flow;
  track: Track;
  ticks: TickConfig;
  slots: Slots;
  tool: Tool;
  snapDuration: NoteDuration;
};

export const SequencerTrack: FC<SequencerTrackProps> = ({
  flow,
  track,
  ticks,
  slots,
  tool,
  snapDuration,
}) => {
  const sequencerRef = useRef<HTMLDivElement>(null);

  const onPointerDown = useCallback(
    async (e: PointerEvent<HTMLDivElement>) => {
      if (tool !== Tool.Write) {
        return;
      }

      const box = sequencerRef.current?.getBoundingClientRect();
      if (!box) {
        return;
      }

      e.stopPropagation();

      const ticksPerSnap = (flow.subdivisions * 4) / snapDuration;
      const x = e.clientX - box.left;
      const y = e.clientY - box.top;

      const pitch = getPitchFromYPosition(y, slots.slotHeight);
      const start = getSnappedXPosition(
        x,
        ticks.tickWidth,
        ticksPerSnap,
        "down"
      );

      const key = await actions.score.tone.create(
        track.id,
        start,
        ticksPerSnap,
        pitch,
        null,
        255,
        Articulation.None
      );

      // TODO:
      // audition(instrumentKey, pitch);

      const handlePointerMove = (e: any) => {
        const x = e.clientX - box.left;

        const end = getSnappedXPosition(x, ticks.tickWidth, ticksPerSnap, "up");

        console.log(start, "->", end);

        // if (end > data.start) {
        //   core.set_tone_duration(trackKey, data.toneKey, end - data.start);
        // }
      };

      const handlePointerUp = () => {
        document.removeEventListener("pointermove", handlePointerMove);
        document.removeEventListener("pointerup", handlePointerUp);
      };

      document.addEventListener("pointermove", handlePointerMove);
      document.addEventListener("pointerup", handlePointerUp);
    },
    [
      flow.subdivisions,
      slots.slotHeight,
      snapDuration,
      ticks.tickWidth,
      tool,
      track.id,
    ]
  );

  return (
    <div
      ref={sequencerRef}
      onPointerDown={onPointerDown}
      className="ignore-x relative"
      style={{ width: ticks.length * ticks.tickWidth, height: slots.height }}
    >
      {slots.pattern.map((slot, i) => {
        return (
          <div
            key={i}
            className="absolute top-0 left-0 w-full"
            style={{
              translate: `0 ${slots.slotHeight * i}px`,
              height: slots.slotHeight - 1,
              backgroundColor:
                slot.type === KeyType.Black ? "#1c1c1c" : "#1f1f1f",
            }}
          />
        );
      })}

      {ticks.entries.map((entry, tick) => {
        if (entry.type === "sub") {
          return null;
        }

        return (
          <div
            key={tick}
            className={classNames("absolute top-0 left-0", {
              "bg-neutral-500": entry.type === "bar",
              "bg-neutral-800": entry.type === "beat",
            })}
            style={{
              height: slots.height,
              width: 1,
              translate: `${entry.x}px`,
            }}
          />
        );
      })}
    </div>
  );
};
