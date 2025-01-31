import { FC, PointerEvent, useCallback, useRef } from "react";
import { TickConfig } from "../hooks/use-ticks";
import { KeyType, Slots } from "../hooks/use-slots";
import classNames from "classnames";
import { Tool } from "../../../services/types";

type SequencerTrackProps = {
  ticks: TickConfig;
  slots: Slots;
  tool: Tool;
};

export const SequencerTrack: FC<SequencerTrackProps> = ({
  ticks,
  slots,
  tool,
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

      const x = e.clientX - box.left;
      const y = e.clientY - box.top;

      console.log(x, y);

      // const tick = getTickFromXPosition(x, ticks.tick_width);
      // const pitch = getPitchFromYPosition(y, slots.slot_height);
      // const start = getSnappedTick(tick, ticks.ticks_per_snap, "down");

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

        console.log(x);
        // const tick = getTickFromXPosition(x, ticks.tick_width);
        // const end = getSnappedTick(tick, ticks.ticks_per_snap, "up");

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
    [tool]
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
