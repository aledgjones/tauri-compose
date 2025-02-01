import { FC, PointerEvent, useCallback, useRef } from "react";
import { TickConfig } from "../hooks/use-ticks";
import { KeyType, Slots } from "../hooks/use-slots";
import classNames from "classnames";
import { Flow, NoteDuration, Tool } from "../../../services/types";

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

type SequencerTrackProps = {
  flow: Flow;
  ticks: TickConfig;
  slots: Slots;
  tool: Tool;
  snapDuration: NoteDuration;
};

export const SequencerTrack: FC<SequencerTrackProps> = ({
  flow,
  ticks,
  slots,
  tool,
  snapDuration,
}) => {
  const sequencerRef = useRef<HTMLDivElement>(null);

  const onPointerDown = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      if (tool !== Tool.Write) {
        return;
      }

      console.log("DOWN");

      const box = sequencerRef.current?.getBoundingClientRect();
      if (!box) {
        return;
      }

      e.stopPropagation();

      const ticksPerSnap = (flow.subdivisions * 4) / snapDuration;
      const x = e.clientX - box.left;
      const y = e.clientY - box.top;

      // const pitch = getPitchFromYPosition(y, slots.slot_height);
      const start = getSnappedXPosition(
        x,
        ticks.tickWidth,
        ticksPerSnap,
        "down"
      );

      // const key = core.create_tone(
      //   trackKey,
      //   start,
      //   ticks.ticks_per_snap,
      //   pitch,
      //   undefined,
      //   100,
      //   Articulation.None
      // );

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
    [flow.subdivisions, snapDuration, ticks.tickWidth, tool]
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
