import { useMemo } from "react";
import { useAppState } from "../../../services/state";
import { SelectionType } from "../../../services/types";
import { getPitchName } from "../../../services/pitch-utils";

const ALPHABET = ["A", "B"];
const TRACK_LABEL = ["Up-stem", "Down-stem"];

export const useSelectedToneInfo = () => {
  const state = useAppState();

  const selectedToneInfo = useMemo(() => {
    const selection = state.ui.sequencer.selection;
    if (!selection || selection.selection_type !== SelectionType.Tone) {
      return null;
    }

    const track = state.score.tracks.by_id[selection.track_id];
    if (!track) {
      return null;
    }

    // Find the tone in the track
    let tone = null;
    for (const tick of Object.keys(track.entries.by_tick)) {
      const entryIds = track.entries.by_tick[parseInt(tick)];
      for (const entryId of entryIds) {
        const entry = track.entries.by_id[entryId];
        if (entry.type === "Tone" && entry.id === selection.id) {
          tone = entry;
          break;
        }
      }
      if (tone) break;
    }

    if (!tone) {
      return null;
    }

    // Get note name
    const noteName = getPitchName(tone.pitch);

    // Find which instrument owns this track by checking staves in the flow
    const flow = state.score.flows.by_id[selection.flow_id];
    let instrument = null;

    if (flow) {
      // Search through all instruments to find which one has this track
      for (const playerId of state.score.players.order) {
        const player = state.score.players.by_id[playerId];
        for (const instrumentId of player.instrument_ids) {
          const inst = state.score.instruments.by_id[instrumentId];
          if (!inst) continue;

          // Check if any of this instrument's staves contain the track
          for (const staveId of inst.stave_ids) {
            const stave = flow.staves.by_id[staveId];
            if (stave && stave.track_ids.includes(selection.track_id)) {
              instrument = inst;
              break;
            }
          }
          if (instrument) break;
        }
        if (instrument) break;
      }
    }

    const instrumentName = instrument
      ? instrument.long_name + (instrument.count ? ` ${instrument.count}` : "")
      : "Unknown";

    // Build track label from the selected flow and track
    let trackLabel = "Unknown";

    if (flow && instrument) {
      const stave_ids = instrument.stave_ids;

      for (let i = 0; i < stave_ids.length; i++) {
        const stave = flow.staves.by_id[stave_ids[i]];
        if (!stave) continue;

        const trackIndex = stave.track_ids.indexOf(selection.track_id);
        if (trackIndex !== -1) {
          const staffLabel =
            stave_ids.length > 1 ? `Staff (${ALPHABET[i]}), ` : ``;
          const voiceLabel = `${TRACK_LABEL[trackIndex % 2]} voice ${
            Math.floor(trackIndex / 2) + 1
          }`;
          trackLabel = staffLabel + voiceLabel;
          break;
        }
      }
    }

    return {
      noteName,
      instrumentName,
      trackLabel,
    };
  }, [
    state.ui.sequencer.selection,
    state.score.tracks.by_id,
    state.score.instruments.by_id,
    state.score.flows.by_id,
    state.score.players.order,
    state.score.players.by_id,
  ]);

  return selectedToneInfo;
};
