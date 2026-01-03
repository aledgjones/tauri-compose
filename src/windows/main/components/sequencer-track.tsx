import { FC, PointerEvent, useCallback, useMemo, useRef } from "react";
import { TickConfig } from "../hooks/use-ticks";
import { KeyType, Slots } from "../hooks/use-slots";
import classNames from "classnames";
import {
  Articulation,
  Flow,
  NoteDuration,
  SelectionType,
  Tool,
  Track,
  UniqueId,
} from "../../../services/types";
import { actions } from "../../../services/actions";
import { SequencerTrackTone } from "./sequencer-track-tone";
import { Expression, sampler } from "../../../sampler";
import { useAppState } from "../../../services/state";
import { getSequencerClickTarget } from "../../../services/sequencer-utils";

export const getSnappedXPosition = (
  x: number,
  tickWidth: number,
  ticksPerSnap: number,
  direction: "up" | "down" | "nearest",
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
  instrumentId: UniqueId | null;
  color: string;
};

export const SequencerTrack: FC<SequencerTrackProps> = ({
  flow,
  track,
  ticks,
  slots,
  tool,
  snapDuration,
  instrumentId,
  color,
}) => {
  const sequencerRef = useRef<HTMLDivElement>(null);
  const appState = useAppState();

  const onPointerDown = useCallback(
    async (e: PointerEvent<HTMLDivElement>) => {
      switch (tool) {
        case Tool.Write: {
          const box = sequencerRef.current?.getBoundingClientRect();

          if (!box) {
            return;
          }

          e.stopPropagation();

          const ticksPerSnap = (flow.subdivisions * 4) / snapDuration;
          const x = e.clientX - box.left;
          const y = e.clientY - box.top;
          let duration = ticksPerSnap;

          const pitch = getPitchFromYPosition(y, slots.slotHeight);
          const start = getSnappedXPosition(
            x,
            ticks.tickWidth,
            ticksPerSnap,
            "down",
          );

          const tone_id = await actions.score.tone.create(
            track.id,
            start,
            ticksPerSnap,
            pitch,
            null,
            255,
            Articulation.None,
          );

          // Select the newly created tone
          await actions.ui.sequencer.select(SelectionType.Tone, tone_id);

          // Audition the tone
          if (instrumentId) {
            sampler.play(
              instrumentId.toString(),
              Expression.Natural,
              pitch,
              sampler.ctx.currentTime,
              0.5,
            );
          }

          const handlePointerMove = async (e: any) => {
            const x = e.clientX - box.left;
            const end = getSnappedXPosition(
              x,
              ticks.tickWidth,
              ticksPerSnap,
              "up",
            );
            const newDuration = end - start;

            if (end > start && duration !== newDuration) {
              duration = newDuration;
              await actions.score.tone.setDuration(
                track.id,
                tone_id,
                end - start,
              );
            }
          };

          const handlePointerUp = () => {
            document.removeEventListener("pointermove", handlePointerMove);
            document.removeEventListener("pointerup", handlePointerUp);
          };

          document.addEventListener("pointermove", handlePointerMove);
          document.addEventListener("pointerup", handlePointerUp);
          break;
        }

        case Tool.Select: {
          const clickTarget = getSequencerClickTarget(e.target as HTMLElement);

          if (clickTarget === null) {
            await actions.ui.sequencer.clearSelection();
            break;
          }

          await actions.ui.sequencer.select(SelectionType.Tone, clickTarget.id);

          break;
        }

        case Tool.Erase: {
          const clickTarget = getSequencerClickTarget(e.target as HTMLElement);

          if (clickTarget === null) {
            break;
          }

          await actions.score.tone.remove(track.id, clickTarget.id);
          await actions.ui.sequencer.clearSelection();

          break;
        }
      }
    },
    [
      flow.subdivisions,
      slots.slotHeight,
      snapDuration,
      ticks.tickWidth,
      tool,
      track.id,
      instrumentId,
      appState.ui.sequencer.selection,
    ],
  );

  const tones = useMemo(() => {
    return Object.values(track.entries.by_id).filter(
      (entry) => entry.type === "Tone",
    );
  }, [track.entries.by_id]);

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

      {tones.map((tone) => {
        return (
          <SequencerTrackTone
            key={tone.id}
            slotHeight={slots.slotHeight}
            tickWidth={ticks.tickWidth}
            tone={tone}
            color={color}
          />
        );
      })}
    </div>
  );
};
