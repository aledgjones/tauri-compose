import { FC, useMemo } from "react";
import { Instrument, Flow, Tone, Tuplet } from "../../../../../services/types";
import { TickConfig } from "../../../hooks/use-ticks";
import { useAppState } from "../../../../../services/state";
import classNames from "classnames";
import {
  AxisAddress,
  buildSequencerAddressLattice,
  getSpanForAddressDuration,
} from "../../../components/sequencer-address-lattice";

const range = { min: 21, max: 108 };
const rowHeight = 96;
const pianoRangeSize = range.max - range.min;

const getToneAddress = (tone: Tone, tupletId: number | null): AxisAddress => ({
  tuplet_id: tupletId,
  tick: tone.tick,
});

interface MiniaturePianoRollProps {
  instrument: Instrument;
  flow: Flow;
  ticks: TickConfig;
  color: string;
  isDisabled: boolean;
}

export const MiniaturePianoRoll: FC<MiniaturePianoRollProps> = ({
  instrument,
  flow,
  ticks,
  color,
  isDisabled,
}) => {
  const state = useAppState();

  // Get all tracks for this instrument
  const tracks = useMemo(() => {
    return instrument.stave_ids.flatMap((staveId) => {
      const stave = flow.staves.by_id[staveId];
      if (!stave) return [];

      return stave.track_ids
        .map((trackId) => {
          return state.score.tracks.by_id[trackId];
        })
        .filter(Boolean);
    });
  }, [instrument.stave_ids, flow.staves.by_id, state.score.tracks.by_id]);

  const renderedTones = useMemo(() => {
    const allTones: Array<{
      id: number;
      pitch: number;
      leftPx: number;
      widthPx: number;
      tupletId: number | null;
    }> = [];

    tracks.forEach((track) => {
      if (!track) {
        return;
      }

      const trackTuplets = Object.values(track.entries.by_id).filter(
        (entry) => entry.type === "Tuplet",
      ) as Tuplet[];
      const addressLattice = buildSequencerAddressLattice(
        flow.length,
        ticks.tickWidth,
        trackTuplets,
      );

      Object.values(track.entries.by_id).forEach((entry) => {
        if (entry.type !== "Tone") {
          return;
        }

        const span = getSpanForAddressDuration(
          addressLattice,
          getToneAddress(entry, null),
          entry.duration,
        );

        if (!span) {
          return;
        }

        allTones.push({
          id: entry.id,
          pitch: entry.pitch.int,
          leftPx: span.startX,
          widthPx: span.width,
          tupletId: null,
        });
      });

      trackTuplets.forEach((tuplet) => {
        Object.values(tuplet.entries.by_id).forEach((entry) => {
          const span = getSpanForAddressDuration(
            addressLattice,
            getToneAddress(entry, tuplet.id),
            entry.duration,
          );

          if (!span) {
            return;
          }

          allTones.push({
            id: entry.id,
            pitch: entry.pitch.int,
            leftPx: span.startX,
            widthPx: span.width,
            tupletId: tuplet.id,
          });
        });
      });
    });

    return allTones;
  }, [tracks, flow.length, ticks.tickWidth]);

  // Merge overlapping time ranges where there are notes
  const activeRanges = useMemo(() => {
    if (renderedTones.length === 0) return [];

    const ranges = renderedTones.map((tone) => ({
      start: tone.leftPx,
      end: tone.leftPx + tone.widthPx,
    }));

    ranges.sort((a, b) => a.start - b.start);

    const merged: Array<{ start: number; end: number }> = [];
    let current = ranges[0];

    for (let i = 1; i < ranges.length; i++) {
      if (ranges[i].start <= current.end) {
        // Overlapping or adjacent, merge
        current.end = Math.max(current.end, ranges[i].end);
      } else {
        // No overlap, add current and start new
        merged.push(current);
        current = ranges[i];
      }
    }
    merged.push(current);

    return merged;
  }, [renderedTones]);

  return (
    <div
      className={classNames("relative bg-neutral-900", {
        "opacity-50": isDisabled,
      })}
      style={{
        height: rowHeight,
        width: ticks.length * ticks.tickWidth,
      }}
    >
      {ticks.entries.map((entry, tick) => {
        if (entry.type !== "bar") {
          return null;
        }

        return (
          <div
            key={tick}
            className="absolute top-0 left-0 h-full bg-neutral-800"
            style={{
              width: 1,
              translate: `${entry.x}px`,
            }}
          />
        );
      })}

      {/* Background highlights for merged time ranges */}
      {activeRanges.map((range, index) => {
        return (
          <div
            key={`bg-${index}`}
            className="absolute opacity-20"
            style={{
              left: range.start,
              top: 0,
              width: Math.max(1, range.end - range.start),
              height: rowHeight,
              backgroundColor: color,
            }}
          />
        );
      })}

      {/* Individual note boxes */}
      {renderedTones.map((tone) => {
        // Position pitch relative to full piano range
        const pitchRatio = (tone.pitch - range.min) / pianoRangeSize;
        const y = (1 - pitchRatio) * (rowHeight - 3);

        return (
          <div
            key={
              tone.tupletId === null ? tone.id : `${tone.tupletId}-${tone.id}`
            }
            className="absolute opacity-80"
            style={{
              left: tone.leftPx,
              top: y,
              width: Math.max(1, tone.widthPx),
              height: 3,
              backgroundColor: color,
            }}
            data-tone-id={tone.id}
            {...(tone.tupletId !== null
              ? { "data-tuplet-id": tone.tupletId }
              : {})}
          />
        );
      })}
    </div>
  );
};
