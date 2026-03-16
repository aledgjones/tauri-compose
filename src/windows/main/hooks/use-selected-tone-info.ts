import { useMemo } from "react";
import { useAppState } from "../../../services/state";
import { getPitchName } from "../../../services/pitch-utils";
import { Tone, Track } from "../../../services/types";

const ALPHABET = ["A", "B"];
const TRACK_LABEL = ["Up-stem", "Down-stem"];

export const getSelectedTone = (
  track: Track,
  toneId: number,
  tupletId?: number | null,
): Tone | null => {
  if (tupletId !== undefined && tupletId !== null) {
    const tupletEntry = track.entries.by_id[tupletId];

    if (tupletEntry && tupletEntry.type === "Tuplet") {
      return tupletEntry.entries.by_id[toneId] ?? null;
    }
  }

  const entry = track.entries.by_id[toneId];
  if (entry && entry.type === "Tone") {
    return entry;
  }

  for (const tick of Object.keys(track.entries.by_tick)) {
    const entryIds = track.entries.by_tick[parseInt(tick)];
    for (const entryId of entryIds) {
      const nestedEntry = track.entries.by_id[entryId];
      if (nestedEntry.type === "Tone" && nestedEntry.id === toneId) {
        return nestedEntry;
      }
    }
  }

  return null;
};

export const useSelectedToneInfo = () => {
  const state = useAppState();

  const selectedToneInfo = useMemo(() => {
    const selection = state.ui.sequencer.selection;

    if (selection === null) {
      return null;
    }

    if (
      selection.type === "Tempo" ||
      selection.type === "TimeSignature" ||
      selection.type === "KeySignature" ||
      selection.type === "Clef" ||
      selection.type === "Tuplet"
    ) {
      return null;
    }

    const track = state.score.tracks.by_id[selection.track_id];

    if (!track) {
      return null;
    }

    // Get instrument from stored instrument_id
    const instrument = state.score.instruments.by_id[selection.instrument_id];
    const instrumentName = instrument
      ? instrument.long_name + (instrument.count ? ` ${instrument.count}` : "")
      : "Unknown";

    // Build track label from the selected flow and track
    const flow = state.score.flows.by_id[selection.flow_id];
    let trackLabel = "Unknown";

    if (flow && instrument) {
      const stave = flow.staves.by_id[selection.stave_id];
      if (stave) {
        const staveIndex = instrument.stave_ids.indexOf(selection.stave_id);
        const trackIndex = stave.track_ids.indexOf(selection.track_id);

        if (trackIndex !== -1) {
          const staffLabel =
            instrument.stave_ids.length > 1
              ? `Staff (${ALPHABET[staveIndex]}), `
              : ``;
          const voiceLabel = `${TRACK_LABEL[trackIndex % 2]} voice ${
            Math.floor(trackIndex / 2) + 1
          }`;
          trackLabel = staffLabel + voiceLabel;
        }
      }
    }

    if (selection.type === "Rest") {
      return {
        noteName: "Rest",
        instrumentName,
        trackLabel,
      };
    }

    if (selection.type === "Tone") {
      const tone = getSelectedTone(
        track,
        selection.tone_id,
        selection.tuplet_id,
      );

      if (!tone) {
        return null;
      }

      // Get note name
      const noteName = getPitchName(tone.pitch);

      return {
        noteName,
        instrumentName,
        trackLabel,
      };
    }
  }, [
    state.ui.sequencer.selection,
    state.score.tracks.by_id,
    state.score.instruments.by_id,
    state.score.flows.by_id,
  ]);

  return selectedToneInfo;
};
