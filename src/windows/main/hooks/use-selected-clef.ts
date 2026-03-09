import { useMemo } from "react";
import { useAppState } from "../../../services/state";

export const useSelectedClef = () => {
  const state = useAppState();

  return useMemo(() => {
    const selection = state.ui.sequencer.selection;

    if (selection === null || selection.type !== "Clef") {
      return null;
    }

    const flow = state.score.flows.by_id[selection.flow_id];
    if (!flow) {
      return null;
    }

    const stave = flow.staves.by_id[selection.stave_id];
    if (!stave) {
      return null;
    }

    const masterTrack = state.score.tracks.by_id[stave.master_id];
    if (!masterTrack) {
      return null;
    }

    const entry = masterTrack.entries.by_id[selection.clef_id];
    if (!entry || entry.type !== "Clef" || entry.tick !== selection.tick) {
      return null;
    }

    return {
      flow_id: selection.flow_id,
      stave_id: selection.stave_id,
      clef: entry,
    };
  }, [
    state.score.flows.by_id,
    state.score.tracks.by_id,
    state.ui.sequencer.selection,
  ]);
};
