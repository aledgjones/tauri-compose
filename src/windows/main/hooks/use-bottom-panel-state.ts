import { useMemo } from "react";
import { useAppState } from "../../../services/state";
import { SelectOption } from "../../../ui/select";
import { UniqueId, Instrument } from "../../../services/types";
import { getFlowName } from "../../../services/utils";

const ALPHABET = ["A", "B"];
const TRACK_LABEL = ["Up-stem", "Down-stem"];

export const useBottomPanelState = () => {
  const state = useAppState();

  const instruments = useMemo(() => {
    return state.score.players.order.flatMap((playerId: UniqueId) => {
      const player = state.score.players.by_id[playerId];
      return player.instrument_ids.map((instrumentId: UniqueId) => {
        return state.score.instruments.by_id[instrumentId];
      });
    });
  }, [state]);

  const instrumentOptions: SelectOption<UniqueId>[] = useMemo(() => {
    return instruments.map((instrument: Instrument) => ({
      value: instrument.id,
      label:
        instrument.long_name + (instrument.count ? ` ${instrument.count}` : ""),
    }));
  }, [instruments]);

  const flowOptions: SelectOption<UniqueId>[] = useMemo(() => {
    return state.score.flows.order.flatMap((candidateFlowId, index) => {
      if (!state.score.flows.by_id[candidateFlowId]) {
        return [];
      }

      return {
        value: candidateFlowId,
        label: getFlowName(state, index, candidateFlowId),
      };
    });
  }, [state]);

  const flowId = useMemo(() => {
    if (state.score.flows.order.length === 0) {
      return null;
    }

    const id = state.ui.flow_id || state.score.flows.order[0] || null;

    if (id && state.score.flows.by_id[id]) {
      return id;
    }

    return state.score.flows.order[0] || null;
  }, [state]);

  const instrumentId = useMemo(() => {
    if (instruments.length === 0) {
      return null;
    }

    const id = state.ui.sequencer.instrument;

    // check the selection still exists
    if (
      id &&
      instruments.some((instrument: Instrument) => instrument.id === id)
    ) {
      return id;
    }

    // fallback to the first instrument
    return instruments[0].id;
  }, [instruments, state.ui.sequencer.instrument]);

  const tracks = useMemo(() => {
    if (!flowId || !instrumentId) {
      return [];
    }

    const flow = state.score.flows.by_id[flowId];
    const instrument = state.score.instruments.by_id[instrumentId];

    if (!flow || !instrument) {
      return [];
    }

    const stave_ids = instrument.stave_ids;

    return stave_ids.flatMap((staveId: UniqueId, i: number) => {
      const stave = flow.staves.by_id[staveId];

      if (!stave) {
        return [];
      }

      return stave.track_ids.map((trackId: UniqueId, ii: number) => {
        const staffLabel =
          stave_ids.length > 1 ? `Staff (${ALPHABET[i]}), ` : ``;
        const trackLabel = `${TRACK_LABEL[ii % 2]} voice ${
          Math.floor(ii / 2) + 1
        }`;

        return {
          value: trackId,
          label: [staffLabel, trackLabel].join(""),
        };
      });
    });
  }, [
    flowId,
    instrumentId,
    state.score.flows.by_id,
    state.score.instruments.by_id,
  ]);

  const trackId = useMemo(() => {
    if (tracks.length === 0) {
      return null;
    }

    const id = state.ui.sequencer.track;

    // check the selection still exists
    if (
      id &&
      tracks.some((option: SelectOption<UniqueId>) => option.value === id)
    ) {
      return id;
    }

    // fallback to the first instrument
    return tracks[0].value;
  }, [state.ui.sequencer.track, tracks]);

  const selectedInstrument = useMemo(() => {
    if (!instrumentId) {
      return null;
    }

    return state.score.instruments.by_id[instrumentId] ?? null;
  }, [instrumentId, state.score.instruments.by_id]);

  const selectedAssignment = useMemo(() => {
    if (!instrumentId) {
      return null;
    }

    return (
      state.resolved_playback_assignment_by_instrument_id[instrumentId] ?? null
    );
  }, [instrumentId, state.resolved_playback_assignment_by_instrument_id]);

  return {
    instruments,
    instrumentOptions,
    flowOptions,
    flowId,
    instrumentId,
    tracks,
    trackId,
    selectedInstrument,
    selectedAssignment,
    tool: state.ui.sequencer.tool,
    snapValue: state.ui.sequencer.snap,
  };
};
