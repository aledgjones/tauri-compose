import { FC, useMemo } from "react";
import { Instrument, Flow } from "../../../../../services/types";
import { TickConfig } from "../../../hooks/use-ticks";
import { useAppState } from "../../../../../services/state";
import classNames from "classnames";

interface MiniaturePianoRollProps {
  instrument: Instrument;
  flow: Flow;
  ticks: TickConfig;
  color: string;
  isInSelectedFlow: boolean;
}

export const MiniaturePianoRoll: FC<MiniaturePianoRollProps> = ({
  instrument,
  flow,
  ticks,
  color,
  isInSelectedFlow,
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
      id: string;
      tick: number;
      duration: number;
      pitch: number;
    }> = [];

    tracks.forEach((track) => {
      if (!track) return;

      Object.values(track.entries.by_id).forEach((entry) => {
        if (entry.type === "Tone") {
          const tone = entry as any; // Type assertion for Tone
          allTones.push({
            id: tone.id,
            tick: tone.tick,
            duration: tone.duration,
            pitch: tone.pitch.int,
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

  // Use full piano range for consistent vertical positioning
  const fullPianoRange = useMemo(
    () => ({
      min: 21, // A0
      max: 108, // C8
    }),
    [],
  );

  const rowHeight = 64; // Match the instrument row height
  const pianoRangeSize = fullPianoRange.max - fullPianoRange.min;

  return (
    <div
      className={classNames("relative bg-neutral-900", {
        "opacity-50": !isInSelectedFlow,
      })}
      style={{
        height: rowHeight,
        width: ticks.length * ticks.tickWidth,
      }}
    >
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
        const pitchRatio =
          pianoRangeSize > 0
            ? (tone.pitch - fullPianoRange.min) / pianoRangeSize
            : 0.5;
        const y = (1 - pitchRatio) * (rowHeight - 3); // 3px for note height

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
