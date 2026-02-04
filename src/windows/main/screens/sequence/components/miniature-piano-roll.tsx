import { FC, useMemo } from "react";
import { Instrument, Flow } from "../../../../../services/types";
import { TickConfig } from "../../../hooks/use-ticks";
import { useAppState } from "../../../../../services/state";
import classNames from "classnames";

const range = { min: 21, max: 108 };
const rowHeight = 96;
const pianoRangeSize = range.max - range.min;

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

  // Collect all tones from all tracks
  const tones = useMemo(() => {
    const allTones: Array<{
      id: number;
      tick: number;
      duration: number;
      pitch: number;
    }> = [];

    tracks.forEach((track) => {
      if (!track) return;

      Object.values(track.entries.by_id).forEach((entry) => {
        if (entry.type === "Tone") {
          allTones.push({
            id: entry.id,
            tick: entry.tick,
            duration: entry.duration,
            pitch: entry.pitch.int,
          });
        }
      });
    });

    return allTones;
  }, [tracks]);

  // Merge overlapping time ranges where there are notes
  const activeRanges = useMemo(() => {
    if (tones.length === 0) return [];

    // Create ranges from tones
    const ranges = tones.map((tone) => ({
      start: tone.tick,
      end: tone.tick + tone.duration,
    }));

    // Sort by start time
    ranges.sort((a, b) => a.start - b.start);

    // Merge overlapping ranges
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
  }, [tones]);

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
        const x = range.start * ticks.tickWidth;
        const width = (range.end - range.start) * ticks.tickWidth;

        return (
          <div
            key={`bg-${index}`}
            className="absolute opacity-20"
            style={{
              left: x,
              top: 0,
              width: Math.max(1, width),
              height: rowHeight,
              backgroundColor: color,
            }}
          />
        );
      })}

      {/* Individual note boxes */}
      {tones.map((tone) => {
        const x = tone.tick * ticks.tickWidth;
        const width = tone.duration * ticks.tickWidth;

        // Position pitch relative to full piano range
        const pitchRatio = (tone.pitch - range.min) / pianoRangeSize;
        const y = (1 - pitchRatio) * (rowHeight - 3);

        return (
          <div
            key={tone.id}
            className="absolute opacity-80"
            style={{
              left: x,
              top: y,
              width: Math.max(1, width),
              height: 3,
              backgroundColor: color,
            }}
          />
        );
      })}
    </div>
  );
};
