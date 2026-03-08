import { useMemo } from "react";
import { useAppState } from "../../../services/state";

export const useSelectedTempo = () => {
  const state = useAppState();

  return useMemo(() => {
    const selection = state.ui.sequencer.selection;

    if (selection === null || selection.type !== "Tempo") {
      return null;
    }

    const flow = state.score.flows.by_id[selection.flow_id];
    if (!flow) {
      return null;
    }

    const masterTrack = state.score.tracks.by_id[flow.master_id];
    if (!masterTrack) {
      return null;
    }

    const entry = masterTrack.entries.by_id[selection.tempo_id];
    if (!entry || entry.type !== "Tempo" || entry.tick !== selection.tick) {
      return null;
    }

    return {
      flow_id: selection.flow_id,
      tempo: entry,
    };
  }, [
    state.score.flows.by_id,
    state.score.tracks.by_id,
    state.ui.sequencer.selection,
  ]);
};
