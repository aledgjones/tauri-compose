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
import { SequencerTrackTone } from "./sequencer-track-tone";
import { SequencerTrackTuplet } from "./sequencer-track-tuplet";
import {
  getSequencerClickTarget,
  ClickTarget,
} from "../../../services/sequencer-utils";
import {
  AddressLookupDirection,
  AxisAddress,
  SequencerAddressLattice,
  SequencerSnapOverlay,
  addressKey,
  advanceIndex,
  buildSequencerAddressLattice,
  buildSnapOverlay,
  findSnappedAddressForX,
  getAddressAtIndex,
  getIndexForAddress,
  getSpanForAddressDuration,
  getXForAddress,
} from "./sequencer-address-lattice";
import {
  getTupletLocalTickFromClientX,
  getTupletRegion,
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

const getTicksPerSnap = (snapDuration: NoteDuration) => {
  const subdivisions = 32;

  return (subdivisions * 4) / snapDuration;
};

const getToneAddress = (
  tone: Tone,
  tupletId: UniqueId | null,
): AxisAddress => ({
  tuplet_id: tupletId,
  tick: tone.tick,
});

const clampIndex = (value: number, min: number, max: number) => {
  return Math.min(max, Math.max(min, value));
};

const getSnappedAddress = (
  overlay: SequencerSnapOverlay,
  x: number,
  direction: AddressLookupDirection = "nearest",
) => {
  return findSnappedAddressForX(overlay, x, direction);
};

const syncSelectedToneOwner = async (
  flow: Flow,
  track: Track,
  instrumentId: UniqueId,
  toneId: UniqueId,
  tupletId: UniqueId | null,
) => {
  const staveId = getStaveIdForTrack(flow, track.id);

  if (staveId === null) {
    return;
  }

  await actions.ui.sequencer.selectTone(
    toneId,
    track.id,
    staveId,
    flow.id,
    instrumentId,
    tupletId ?? undefined,
  );
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
  lattice: SequencerAddressLattice,
  overlay: SequencerSnapOverlay,
  slots: Slots,
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

  const ticksPerSnap = overlay.ticksPerSnap;
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
  const startAddress: AxisAddress = {
    tuplet_id: targetTuplet?.id ?? null,
    tick: startTick,
  };
  const startIndex = getIndexForAddress(lattice, startAddress);

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

  if (startIndex === null) {
    return;
  }

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

  const handlePointerMove = async (e: PointerEvent) => {
    const x = e.clientX - box.left;

    if (targetTuplet) {
      const endAddress = getSnappedAddress(overlay, x);

      if (!endAddress) {
        return;
      }

      const endIndex = getIndexForAddress(lattice, endAddress);

      if (endIndex === null) {
        return;
      }

      const clampedEndIndex = clampIndex(
        endIndex,
        startIndex + ticksPerSnap,
        lattice.continuous.length - 1,
      );
      const clampedDuration = clampedEndIndex - startIndex;

      if (clampedDuration !== duration && clampedDuration >= ticksPerSnap) {
        duration = clampedDuration;
        await actions.score.tone.setDuration(
          track.id,
          tone_id,
          clampedDuration,
          targetTuplet.id,
        );
      }

      return;
    }

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
  currentTupletId: UniqueId | null,
  lattice: SequencerAddressLattice,
  overlay: SequencerSnapOverlay,
  ticksPerSnap: number,
) => {
  const startAddress = getToneAddress(originalTone, currentTupletId);
  const startIndex = getIndexForAddress(lattice, startAddress);

  if (startIndex === null) {
    return;
  }

  let lastDuration = originalTone.duration;
  const lastIndex = lattice.continuous.length - 1;

  const handlePointerMove = async (e: PointerEvent) => {
    const x = e.clientX - box.left;
    const endAddress = getSnappedAddress(overlay, x);

    if (!endAddress) {
      return;
    }

    const endIndex = getIndexForAddress(lattice, endAddress);

    if (endIndex === null) {
      return;
    }

    const clampedEndIndex = clampIndex(
      endIndex,
      startIndex + ticksPerSnap,
      lastIndex,
    );
    const clampedDuration = clampedEndIndex - startIndex;

    if (clampedDuration !== lastDuration && clampedDuration >= ticksPerSnap) {
      try {
        await actions.score.tone.setDuration(
          track.id,
          clickTarget.id,
          clampedDuration,
          currentTupletId ?? undefined,
        );
        lastDuration = clampedDuration;
      } catch {
        return;
      }
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
  flow: Flow,
  clickTarget: ClickTarget,
  originalTone: Tone,
  instrumentId: UniqueId,
  currentTupletId: UniqueId | null,
  lattice: SequencerAddressLattice,
  overlay: SequencerSnapOverlay,
  ticksPerSnap: number,
) => {
  const originalStartAddress = getToneAddress(originalTone, currentTupletId);
  const originalStartIndex = getIndexForAddress(lattice, originalStartAddress);

  if (originalStartIndex === null) {
    return;
  }

  const originalEndIndex = advanceIndex(
    lattice,
    originalStartIndex,
    originalTone.duration,
  );

  if (originalEndIndex === null) {
    return;
  }

  let lastAddress = originalStartAddress;
  let lastDuration = originalTone.duration;
  let lastOwnerTupletId = currentTupletId;

  const handlePointerMove = async (e: PointerEvent) => {
    const x = e.clientX - box.left;
    const nextAddress = getSnappedAddress(overlay, x);

    if (!nextAddress) {
      return;
    }

    const nextIndex = getIndexForAddress(lattice, nextAddress);

    if (nextIndex === null) {
      return;
    }

    const clampedStartIndex = clampIndex(
      nextIndex,
      0,
      originalEndIndex - ticksPerSnap,
    );
    const clampedStartAddress = getAddressAtIndex(lattice, clampedStartIndex);

    if (!clampedStartAddress) {
      return;
    }

    const clampedDuration = originalEndIndex - clampedStartIndex;

    if (
      addressKey(clampedStartAddress) !== addressKey(lastAddress) ||
      clampedDuration !== lastDuration
    ) {
      try {
        await actions.score.tone.setPositionAndDuration(
          track.id,
          clickTarget.id,
          clampedStartAddress.tick,
          clampedDuration,
          lastOwnerTupletId ?? undefined,
          clampedStartAddress.tuplet_id ?? undefined,
        );

        if (lastOwnerTupletId !== clampedStartAddress.tuplet_id) {
          await syncSelectedToneOwner(
            flow,
            track,
            instrumentId,
            clickTarget.id,
            clampedStartAddress.tuplet_id,
          );
        }

        lastAddress = clampedStartAddress;
        lastDuration = clampedDuration;
        lastOwnerTupletId = clampedStartAddress.tuplet_id;
      } catch {
        return;
      }
    }
  };

  const handlePointerUp = async () => {
    document.removeEventListener("pointermove", handlePointerMove);
    document.removeEventListener("pointerup", handlePointerUp);

    await actions.score.tone.unifyArticulations(
      track.id,
      clickTarget.id,
      lastOwnerTupletId ?? undefined,
    );
  };

  document.addEventListener("pointermove", handlePointerMove);
  document.addEventListener("pointerup", handlePointerUp);
};

const handlePositionDragging = async (
  e: React.PointerEvent<HTMLDivElement>,
  box: DOMRect,
  track: Track,
  flow: Flow,
  clickTarget: ClickTarget,
  originalTone: Tone,
  instrumentId: UniqueId,
  lattice: SequencerAddressLattice,
  overlay: SequencerSnapOverlay,
  slots: Slots,
  audition: boolean,
  pluginId: string | null,
) => {
  const originalTupletId =
    clickTarget.type === "Tone" ? (clickTarget.tuplet_id ?? null) : null;
  const originalAddress = getToneAddress(originalTone, originalTupletId);
  const originalStartX = getXForAddress(lattice, originalAddress);
  const originalStartIndex = getIndexForAddress(lattice, originalAddress);

  if (originalStartX === null || originalStartIndex === null) {
    return;
  }

  const maxStartIndex = lattice.continuous.length - 1 - originalTone.duration;
  let lastPitch = originalTone.pitch.int;
  let lastAddress = originalAddress;
  let lastOwnerTupletId = originalTupletId;
  const initialX = e.clientX - box.left;
  const initialOffsetX = initialX - originalStartX;

  const handlePointerMove = async (e: PointerEvent) => {
    const x = e.clientX - box.left;
    const y = e.clientY - box.top;
    const newPitch = getPitchFromYPosition(y, slots.slotHeight);
    const snappedAddress = getSnappedAddress(overlay, x - initialOffsetX);

    if (!snappedAddress) {
      return;
    }

    const snappedIndex = getIndexForAddress(lattice, snappedAddress);

    if (snappedIndex === null) {
      return;
    }

    const clampedIndex = clampIndex(snappedIndex, 0, maxStartIndex);
    const clampedAddress = getAddressAtIndex(lattice, clampedIndex);

    if (!clampedAddress) {
      return;
    }

    if (newPitch !== lastPitch) {
      await actions.score.tone.setPitch(
        track.id,
        lastOwnerTupletId,
        clickTarget.id,
        newPitch,
        null,
      );
      if (audition && pluginId) {
        actions.sampler.audition(pluginId, newPitch);
      }
      lastPitch = newPitch;
    }

    if (addressKey(clampedAddress) !== addressKey(lastAddress)) {
      try {
        await actions.score.tone.shift(
          track.id,
          clickTarget.id,
          clampedAddress.tick,
          lastOwnerTupletId ?? undefined,
          clampedAddress.tuplet_id ?? undefined,
        );

        if (lastOwnerTupletId !== clampedAddress.tuplet_id) {
          await syncSelectedToneOwner(
            flow,
            track,
            instrumentId,
            clickTarget.id,
            clampedAddress.tuplet_id,
          );
        }

        lastAddress = clampedAddress;
        lastOwnerTupletId = clampedAddress.tuplet_id;
      } catch {
        return;
      }
    }
  };

  const handlePointerUp = async () => {
    document.removeEventListener("pointermove", handlePointerMove);
    document.removeEventListener("pointerup", handlePointerUp);

    await actions.score.tone.unifyArticulations(
      track.id,
      clickTarget.id,
      lastOwnerTupletId ?? undefined,
    );
  };

  document.addEventListener("pointermove", handlePointerMove);
  document.addEventListener("pointerup", handlePointerUp);
};

const handleSelectTool = async (
  e: React.PointerEvent<HTMLDivElement>,
  sequencerRef: React.RefObject<HTMLDivElement | null>,
  flow: Flow,
  track: Track,
  lattice: SequencerAddressLattice,
  overlay: SequencerSnapOverlay,
  slots: Slots,
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

  const box = sequencerRef.current?.getBoundingClientRect();
  if (!box) {
    return;
  }

  e.stopPropagation();
  e.preventDefault();
  const ticksPerSnap = overlay.ticksPerSnap;

  const originalEntry = getToneEntryFromClickTarget(track, clickTarget);
  if (!originalEntry || originalEntry.type !== "Tone") {
    return;
  }

  const originalTone = originalEntry as Tone;
  const originalTupletId = clickTarget.tuplet_id ?? null;
  const originalSpan = getSpanForAddressDuration(
    lattice,
    getToneAddress(originalTone, originalTupletId),
    originalTone.duration,
  );

  if (!originalSpan) {
    return;
  }

  const toneWidthPx = originalSpan.width;
  const touchTargetWidth = Math.min(10, Math.max(5, toneWidthPx / 2));
  const clickX = e.clientX - box.left;
  const relativeClickX = clickX - originalSpan.startX;
  const isDurationAdjustment = relativeClickX >= toneWidthPx - touchTargetWidth;
  const isLeftSideAdjustment = relativeClickX <= touchTargetWidth;

  if (isDurationAdjustment) {
    await handleDurationAdjustment(
      box,
      track,
      clickTarget,
      originalTone,
      originalTupletId,
      lattice,
      overlay,
      ticksPerSnap,
    );
  } else if (isLeftSideAdjustment) {
    await handleLeftSideAdjustment(
      box,
      track,
      flow,
      clickTarget,
      originalTone,
      instrumentId,
      originalTupletId,
      lattice,
      overlay,
      ticksPerSnap,
    );
  } else {
    await handlePositionDragging(
      e,
      box,
      track,
      flow,
      clickTarget,
      originalTone,
      instrumentId,
      lattice,
      overlay,
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
    return tuplets.flatMap((tuplet) =>
      Object.values(tuplet.entries.by_id).map((tone) => ({
        tone,
        tupletId: tuplet.id,
      })),
    );
  }, [tuplets]);

  const addressLattice = useMemo(() => {
    return buildSequencerAddressLattice(flow.length, ticks.tickWidth, tuplets);
  }, [flow.length, ticks.tickWidth, tuplets]);

  const snapOverlay = useMemo(() => {
    return buildSnapOverlay(addressLattice, getTicksPerSnap(snapDuration));
  }, [addressLattice, snapDuration]);

  const renderedTones = useMemo(() => {
    return tones
      .map((tone) => {
        const span = getSpanForAddressDuration(
          addressLattice,
          getToneAddress(tone, null),
          tone.duration,
        );

        return span ? { tone, tupletId: null, span } : null;
      })
      .filter((tone): tone is NonNullable<typeof tone> => tone !== null);
  }, [addressLattice, tones]);

  const renderedTupletTones = useMemo(() => {
    return tupletTones
      .map(({ tone, tupletId }) => {
        const span = getSpanForAddressDuration(
          addressLattice,
          getToneAddress(tone, tupletId),
          tone.duration,
        );

        return span ? { tone, tupletId, span } : null;
      })
      .filter((tone): tone is NonNullable<typeof tone> => tone !== null);
  }, [addressLattice, tupletTones]);

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
            addressLattice,
            snapOverlay,
            slots,
            instrumentId,
            audition,
            pluginId,
          );
          return;

        case Tool.Select:
          await handleSelectTool(
            e,
            sequencerRef,
            flow,
            track,
            addressLattice,
            snapOverlay,
            slots,
            instrumentId,
            audition,
            pluginId,
          );
          return;

        case Tool.Erase:
          await handleEraseTool(e, track);
          return;

        case Tool.Slice:
          await handleSliceTool(e, sequencerRef, track, ticks, snapDuration);
          return;
      }
    },
    [
      flow,
      addressLattice,
      slots,
      snapOverlay,
      tool,
      track,
      instrumentId,
      ticks,
      snapDuration,
      audition,
      pluginId,
    ],
  );

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

      {renderedTones.map(({ tone, span }) => {
        return (
          <SequencerTrackTone
            key={tone.id}
            slotHeight={slots.slotHeight}
            tickWidth={ticks.tickWidth}
            tone={tone}
            color={color}
            startX={span.startX}
            widthPx={span.width}
          />
        );
      })}

      {renderedTupletTones.map(({ tone, tupletId, span }) => {
        return (
          <SequencerTrackTone
            key={`${tupletId}-${tone.id}`}
            slotHeight={slots.slotHeight}
            tickWidth={ticks.tickWidth}
            tone={tone}
            color={color}
            startX={span.startX}
            widthPx={span.width}
            tupletId={tupletId}
          />
        );
      })}
    </div>
  );
};
