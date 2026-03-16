import { FC, useCallback, useMemo, useRef } from "react";
import { TickConfig } from "../hooks/use-ticks";
import { KeyType, Slots } from "../hooks/use-slots";
import classNames from "classnames";
import {
  Flow,
  NoteDuration,
  Tool,
  Tone,
  Track,
  Tuplet,
  UniqueId,
} from "../../../services/types";
import { actions } from "../../../services/actions";
import { noteDurationToTicks } from "../../../services/utils";
import { SequencerTrackTone } from "./sequencer-track-tone";
import { SequencerTrackTuplet } from "./sequencer-track-tuplet";
import {
  getSequencerClickTarget,
  ClickTarget,
} from "../../../services/sequencer-utils";
import {
  getTupletRegion,
  getTupletHostTicks,
  getTupletLocalTickFromClientX,
  isTickInsideTupletRegion,
} from "./sequencer-tuplet-utils";

const getStaveIdForTrack = (flow: Flow, trackId: UniqueId) => {
  for (const stave of Object.values(flow.staves.by_id)) {
    if (stave.track_ids.includes(trackId)) {
      return stave.id;
    }
  }

  return null;
};

const getSnappedXPosition = (
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

const getPitchFromYPosition = (y: number, slotHeight: number) => {
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

const getToneEntryFromClickTarget = (
  track: Track,
  clickTarget: ClickTarget,
) => {
  if (clickTarget.type !== "Tone") {
    return null;
  }

  if (clickTarget.tuplet_id !== undefined) {
    const tupletEntry = track.entries.by_id[clickTarget.tuplet_id];

    if (!tupletEntry || tupletEntry.type !== "Tuplet") {
      return null;
    }

    return tupletEntry.entries.by_id[clickTarget.id] ?? null;
  }

  return track.entries.by_id[clickTarget.id] ?? null;
};

const getTargetTupletFromClickTarget = (
  track: Track,
  clickTarget: ClickTarget | null,
) => {
  if (!clickTarget) {
    return null;
  }

  const tupletId =
    clickTarget.type === "Tuplet" ? clickTarget.id : clickTarget.tuplet_id;

  if (tupletId === undefined) {
    return null;
  }

  const tupletEntry = track.entries.by_id[tupletId];

  if (!tupletEntry || tupletEntry.type !== "Tuplet") {
    return null;
  }

  return tupletEntry;
};

type SequencerTrackProps = {
  flow: Flow;
  track: Track;
  ticks: TickConfig;
  slots: Slots;
  tool: Tool;
  snapDuration: NoteDuration;
  instrumentId: UniqueId;
  color: string;
  audition: boolean;
  pluginId: string | null;
};

const handleWriteTool = async (
  e: React.PointerEvent<HTMLDivElement>,
  sequencerRef: React.RefObject<HTMLDivElement | null>,
  flow: Flow,
  track: Track,
  ticks: TickConfig,
  slots: Slots,
  snapDuration: NoteDuration,
  instrumentId: UniqueId,
  audition: boolean,
  pluginId: string | null,
) => {
  const box = sequencerRef.current?.getBoundingClientRect();
  if (!box) {
    return;
  }

  e.stopPropagation();
  e.preventDefault();

  const subdivisions = 32;
  const ticksPerSnap = (subdivisions * 4) / snapDuration;
  const x = e.clientX - box.left;
  const y = e.clientY - box.top;
  let duration = ticksPerSnap;

  const pitch = getPitchFromYPosition(y, slots.slotHeight);
  const clickTarget = getSequencerClickTarget(e.target as HTMLElement);
  const targetTuplet = getTargetTupletFromClickTarget(track, clickTarget);
  const clampedStart = targetTuplet
    ? getTupletLocalTickFromClientX(
        x,
        targetTuplet,
        ticks.tickWidth,
        ticksPerSnap,
        getSnappedXPosition,
      )
    : Math.max(
        0,
        Math.min(
          flow.length - ticksPerSnap,
          getSnappedXPosition(x, ticks.tickWidth, ticksPerSnap, "down"),
        ),
      );
  const startTick = clampedStart;

  const tone_id = await actions.score.tone.create(
    track.id,
    clampedStart,
    duration,
    pitch,
    null,
    100,
    [],
    targetTuplet?.id,
  );

  // Audition the created note
  if (audition && pluginId) {
    actions.sampler.audition(pluginId, pitch);
  }

  // Find the stave that contains this track
  let stave_id: UniqueId | null = null;
  for (const stave of Object.values(flow.staves.by_id)) {
    if (stave.track_ids.includes(track.id)) {
      stave_id = stave.id;
      break;
    }
  }

  if (!stave_id) {
    console.error("Could not find stave for track", track.id);
    return;
  }

  // Select the newly created tone
  await actions.ui.sequencer.selectTone(
    tone_id,
    track.id,
    stave_id,
    flow.id,
    instrumentId,
    targetTuplet?.id,
  );

  if (targetTuplet) {
    return;
  }

  const handlePointerMove = async (e: PointerEvent) => {
    const x = e.clientX - box.left;
    const endTick = getSnappedXPosition(
      x,
      ticks.tickWidth,
      ticksPerSnap,
      "nearest",
    );

    const newDuration = Math.max(ticksPerSnap, endTick - startTick);

    // Ensure the tone doesn't extend beyond flow bounds
    const maxEndTick = flow.length;
    const clampedEndTick = Math.min(maxEndTick, startTick + newDuration);
    const clampedDuration = clampedEndTick - startTick;

    if (clampedDuration !== duration && clampedDuration >= ticksPerSnap) {
      duration = clampedDuration;
      await actions.score.tone.setDuration(track.id, tone_id, clampedDuration);
    }
  };

  const handlePointerUp = () => {
    document.removeEventListener("pointermove", handlePointerMove);
    document.removeEventListener("pointerup", handlePointerUp);
  };

  document.addEventListener("pointermove", handlePointerMove);
  document.addEventListener("pointerup", handlePointerUp);
};

const handleSliceTool = async (
  e: React.PointerEvent<HTMLDivElement>,
  sequencerRef: React.RefObject<HTMLDivElement | null>,
  track: Track,
  ticks: TickConfig,
  snapDuration: NoteDuration,
) => {
  const clickTarget = getSequencerClickTarget(e.target as HTMLElement);

  if (clickTarget === null) {
    return;
  }

  if (clickTarget.type !== "Tone") {
    return;
  }

  const box = sequencerRef.current?.getBoundingClientRect();
  if (!box) {
    return;
  }

  e.stopPropagation();
  e.preventDefault();

  const subdivisions = 32;
  const ticksPerSnap = (subdivisions * 4) / snapDuration;
  const x = e.clientX - box.left;

  const sliceTick = getSnappedXPosition(
    x,
    ticks.tickWidth,
    ticksPerSnap,
    "nearest",
  );

  const newToneId = await actions.score.tone.slice(
    track.id,
    clickTarget.id,
    sliceTick,
  );

  if (!newToneId) {
    return;
  }

  await actions.ui.sequencer.clearSelection();
};

const handleEraseTool = async (
  e: React.PointerEvent<HTMLDivElement>,
  track: Track,
) => {
  const clickTarget = getSequencerClickTarget(e.target as HTMLElement);

  if (clickTarget === null) {
    return;
  }

  if (clickTarget.type !== "Tone") {
    return;
  }

  await actions.score.tone.remove(
    track.id,
    clickTarget.tuplet_id ?? null,
    clickTarget.id,
  );
  await actions.ui.sequencer.clearSelection();
};

const handleDurationAdjustment = async (
  box: DOMRect,
  track: Track,
  clickTarget: ClickTarget,
  originalTone: Tone,
  ticks: TickConfig,
  ticksPerSnap: number,
  flow: Flow,
) => {
  let lastDuration = originalTone.duration;

  const handlePointerMove = async (e: PointerEvent) => {
    const x = e.clientX - box.left;
    const endTick = getSnappedXPosition(
      x,
      ticks.tickWidth,
      ticksPerSnap,
      "nearest",
    );

    const startTick = originalTone.tick;
    const newDuration = Math.max(ticksPerSnap, endTick - startTick);

    // Ensure the tone doesn't extend beyond flow bounds
    const maxEndTick = flow.length;
    const clampedEndTick = Math.min(maxEndTick, startTick + newDuration);
    const clampedDuration = clampedEndTick - startTick;

    if (clampedDuration !== lastDuration && clampedDuration >= ticksPerSnap) {
      lastDuration = clampedDuration;
      await actions.score.tone.setDuration(
        track.id,
        clickTarget.id,
        clampedDuration,
      );
    }
  };

  const handlePointerUp = () => {
    document.removeEventListener("pointermove", handlePointerMove);
    document.removeEventListener("pointerup", handlePointerUp);
  };

  document.addEventListener("pointermove", handlePointerMove);
  document.addEventListener("pointerup", handlePointerUp);
};

const handleLeftSideAdjustment = async (
  box: DOMRect,
  track: Track,
  clickTarget: ClickTarget,
  originalTone: Tone,
  ticks: TickConfig,
  ticksPerSnap: number,
  flow: Flow,
) => {
  const originalEndTick = originalTone.tick + originalTone.duration;
  let lastTick = originalTone.tick;
  let lastDuration = originalTone.duration;

  const handlePointerMove = async (e: PointerEvent) => {
    const x = e.clientX - box.left;
    const newStartTick = getSnappedXPosition(
      x,
      ticks.tickWidth,
      ticksPerSnap,
      "nearest",
    );

    // Calculate new duration to maintain end position
    const newDuration = originalEndTick - newStartTick;

    // Don't move if it would make the tone too short or start before 0
    if (newDuration < ticksPerSnap || newStartTick < 0) {
      return;
    }

    // Ensure the tone doesn't extend beyond flow bounds
    const clampedStartTick = Math.max(0, newStartTick);
    const clampedDuration = Math.min(
      flow.length - clampedStartTick,
      newDuration,
    );

    // Update both position and duration in one operation
    if (clampedStartTick !== lastTick || clampedDuration !== lastDuration) {
      await actions.score.tone.setPositionAndDuration(
        track.id,
        clickTarget.id,
        clampedStartTick,
        clampedDuration,
      );
      lastTick = clampedStartTick;
      lastDuration = clampedDuration;
    }
  };

  const handlePointerUp = async () => {
    document.removeEventListener("pointermove", handlePointerMove);
    document.removeEventListener("pointerup", handlePointerUp);

    await actions.score.tone.unifyArticulations(track.id, clickTarget.id);
  };

  document.addEventListener("pointermove", handlePointerMove);
  document.addEventListener("pointerup", handlePointerUp);
};

const handlePositionDragging = async (
  e: React.PointerEvent<HTMLDivElement>,
  box: DOMRect,
  track: Track,
  clickTarget: ClickTarget,
  originalTone: Tone,
  ticks: TickConfig,
  ticksPerSnap: number,
  flow: Flow,
  slots: Slots,
  audition: boolean,
  pluginId: string | null,
) => {
  let lastPitch = originalTone.pitch.int;
  let lastTick = originalTone.tick;
  const tupletId =
    clickTarget.type === "Tone" ? (clickTarget.tuplet_id ?? null) : null;

  // Calculate initial position for relative movement
  const initialX = e.clientX - box.left;
  const initialTickRaw = initialX / ticks.tickWidth;
  const originalToneTick = originalTone.tick;

  const handlePointerMove = async (e: PointerEvent) => {
    const x = e.clientX - box.left;
    const y = e.clientY - box.top;

    // Calculate new pitch (vertical movement)
    const newPitch = getPitchFromYPosition(y, slots.slotHeight);

    // Calculate new tick (horizontal movement) - relative delta movement
    const currentTickRaw = x / ticks.tickWidth;
    const delta = currentTickRaw - initialTickRaw;
    const newToneTickRaw = originalToneTick + delta;

    const newTick = getSnappedXPosition(
      newToneTickRaw * ticks.tickWidth,
      ticks.tickWidth,
      ticksPerSnap,
      "nearest",
    );

    // Ensure bounds after snapping
    const maxTick = flow.length - originalTone.duration;
    const clampedNewTick = Math.max(0, Math.min(maxTick, newTick));

    // Update pitch if it changed
    if (newPitch !== lastPitch) {
      await actions.score.tone.setPitch(
        track.id,
        tupletId,
        clickTarget.id,
        newPitch,
        null,
      );
      if (audition && pluginId) {
        actions.sampler.audition(pluginId, newPitch);
      }
      lastPitch = newPitch;
    }

    // Update tick if it changed
    if (clampedNewTick !== lastTick) {
      await actions.score.tone.shift(track.id, clickTarget.id, clampedNewTick);
      lastTick = clampedNewTick;
    }
  };

  const handlePointerUp = async () => {
    document.removeEventListener("pointermove", handlePointerMove);
    document.removeEventListener("pointerup", handlePointerUp);

    await actions.score.tone.unifyArticulations(track.id, clickTarget.id);
  };

  document.addEventListener("pointermove", handlePointerMove);
  document.addEventListener("pointerup", handlePointerUp);
};

const handleSelectTool = async (
  e: React.PointerEvent<HTMLDivElement>,
  sequencerRef: React.RefObject<HTMLDivElement | null>,
  flow: Flow,
  track: Track,
  ticks: TickConfig,
  slots: Slots,
  snapDuration: NoteDuration,
  instrumentId: UniqueId,
  audition: boolean,
  pluginId: string | null,
) => {
  const clickTarget = getSequencerClickTarget(e.target as HTMLElement);

  if (clickTarget === null || clickTarget.type === "Tuplet") {
    await actions.ui.sequencer.clearSelection();
    return;
  }

  const stave_id = getStaveIdForTrack(flow, track.id);

  if (!stave_id) {
    console.error("Could not find stave for track", track.id);
    return;
  }

  await actions.ui.sequencer.selectTone(
    clickTarget.id,
    track.id,
    stave_id,
    flow.id,
    instrumentId,
    clickTarget.tuplet_id,
  );

  if (clickTarget.tuplet_id !== undefined) {
    return;
  }

  const box = sequencerRef.current?.getBoundingClientRect();
  if (!box) {
    return;
  }

  e.stopPropagation();
  e.preventDefault();

  // Calculate snap settings
  const subdivisions = 32;
  const ticksPerSnap = (subdivisions * 4) / snapDuration;

  // Get the original tone data
  const originalEntry = getToneEntryFromClickTarget(track, clickTarget);
  if (!originalEntry || originalEntry.type !== "Tone") {
    return;
  }

  const originalTone = originalEntry as Tone;

  // Calculate tone width and determine touch target areas
  const toneWidthPx = originalTone.duration * ticks.tickWidth;
  const touchTargetWidth = Math.min(10, Math.max(5, toneWidthPx / 2)); // Max 10px, or half for small tones, min 5px
  const toneStartPx = originalTone.tick * ticks.tickWidth;

  // Get click position relative to tone
  const clickX = e.clientX - box.left;
  const relativeClickX = clickX - toneStartPx;

  // Determine interaction mode based on click position
  const isDurationAdjustment = relativeClickX >= toneWidthPx - touchTargetWidth;
  const isLeftSideAdjustment = relativeClickX <= touchTargetWidth;

  if (isDurationAdjustment) {
    await handleDurationAdjustment(
      box,
      track,
      clickTarget,
      originalTone,
      ticks,
      ticksPerSnap,
      flow,
    );
  } else if (isLeftSideAdjustment) {
    await handleLeftSideAdjustment(
      box,
      track,
      clickTarget,
      originalTone,
      ticks,
      ticksPerSnap,
      flow,
    );
  } else {
    await handlePositionDragging(
      e,
      box,
      track,
      clickTarget,
      originalTone,
      ticks,
      ticksPerSnap,
      flow,
      slots,
      audition,
      pluginId,
    );
  }
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
  audition,
  pluginId,
}) => {
  const sequencerRef = useRef<HTMLDivElement>(null);

  const onPointerDown = useCallback(
    async (e: React.PointerEvent<HTMLDivElement>) => {
      switch (tool) {
        case Tool.Write:
          await handleWriteTool(
            e,
            sequencerRef,
            flow,
            track,
            ticks,
            slots,
            snapDuration,
            instrumentId,
            audition,
            pluginId,
          );
          break;

        case Tool.Select:
          await handleSelectTool(
            e,
            sequencerRef,
            flow,
            track,
            ticks,
            slots,
            snapDuration,
            instrumentId,
            audition,
            pluginId,
          );
          break;

        case Tool.Erase:
          await handleEraseTool(e, track);
          break;

        case Tool.Slice:
          await handleSliceTool(e, sequencerRef, track, ticks, snapDuration);
          break;
      }
    },
    [
      flow,
      slots,
      snapDuration,
      tool,
      track,
      instrumentId,
      ticks,
      audition,
      pluginId,
    ],
  );

  const tones = useMemo(() => {
    return Object.values(track.entries.by_id).filter(
      (entry) => entry.type === "Tone",
    ) as Tone[];
  }, [track.entries.by_id]);

  const tuplets = useMemo(() => {
    return Object.values(track.entries.by_id).filter(
      (entry) => entry.type === "Tuplet",
    ) as Tuplet[];
  }, [track.entries.by_id]);

  const tupletRegions = useMemo(() => {
    return tuplets.map(getTupletRegion);
  }, [tuplets]);

  const tupletTones = useMemo(() => {
    return tuplets.flatMap((tuplet) => {
      const localLengthTicks =
        tuplet.note_count * noteDurationToTicks(tuplet.base_note_value);
      const tickScale = getTupletHostTicks(tuplet) / localLengthTicks;

      return Object.values(tuplet.entries.by_id).map((tone) => ({
        tone,
        tupletId: tuplet.id,
        originTick: tuplet.tick,
        tickScale,
      }));
    });
  }, [tuplets]);

  const barTicks = useMemo(() => {
    return new Set(
      ticks.entries
        .filter((entry) => entry.type === "bar")
        .map((entry) => entry.x / ticks.tickWidth),
    );
  }, [ticks.entries, ticks.tickWidth]);

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

        const tickPosition = entry.x / ticks.tickWidth;
        const isInsideTuplet = tupletRegions.some((tupletRegion) =>
          isTickInsideTupletRegion(tickPosition, tupletRegion),
        );

        if (entry.type !== "bar" && isInsideTuplet) {
          return null;
        }

        return (
          <div
            key={tick}
            className={classNames("absolute top-0 left-0", {
              "bg-neutral-500": entry.type === "bar",
              "bg-neutral-700": entry.type === "beat",
            })}
            data-sequencer-tick-line={entry.type}
            style={{
              height: slots.height,
              width: 1,
              translate: `${entry.x}px`,
            }}
          />
        );
      })}

      {tuplets.map((tuplet) => {
        return (
          <SequencerTrackTuplet
            key={tuplet.id}
            tuplet={tuplet}
            tickWidth={ticks.tickWidth}
            height={slots.height}
            color={color}
            barTicks={barTicks}
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

      {tupletTones.map(({ tone, tupletId, originTick, tickScale }) => {
        return (
          <SequencerTrackTone
            key={`${tupletId}-${tone.id}`}
            slotHeight={slots.slotHeight}
            tickWidth={ticks.tickWidth}
            tone={tone}
            color={color}
            originTick={originTick}
            tickScale={tickScale}
            tupletId={tupletId}
          />
        );
      })}
    </div>
  );
};
