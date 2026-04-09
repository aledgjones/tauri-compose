import { useMemo } from "react";
import { useAppState } from "../../../services/state";

export const useSelectedPlayingTechnique = () => {
  const state = useAppState();

  return useMemo(() => {
    const selection = state.ui.sequencer.selection;

    if (selection === null || selection.type !== "PlayingTechnique") {
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

    const entry = masterTrack.entries.by_id[selection.playing_technique_id];
    if (
      !entry ||
      entry.type !== "PlayingTechnique" ||
      entry.tick !== selection.tick
    ) {
      return null;
    }

    return {
      flow_id: selection.flow_id,
      stave_id: selection.stave_id,
      playing_technique: entry,
    };
  }, [
    state.score.flows.by_id,
    state.score.tracks.by_id,
    state.ui.sequencer.selection,
  ]);
};
