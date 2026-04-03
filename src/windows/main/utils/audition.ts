import { actions } from "../../../services/actions";
import {
  AppState,
  ResolvedPlaybackRouteStatus,
  UniqueId,
} from "../../../services/types";

export const canAuditionInstrument = (
  state: AppState,
  instrumentId: UniqueId,
) => {
  return (
    state.preferences.audition &&
    state.single_instrument_transport_ready === true &&
    state.resolved_playback_route_by_instrument_id[instrumentId]?.status ===
      ResolvedPlaybackRouteStatus.Ready
  );
};

export const auditionInstrumentPitch = async (
  state: AppState,
  instrumentId: UniqueId,
  pitch: number,
) => {
  if (!canAuditionInstrument(state, instrumentId)) {
    return false;
  }

  try {
    await actions.transport.auditionTone(instrumentId, pitch);
    return true;
  } catch (error) {
    console.error("Failed to audition tone", error);
    return false;
  }
};
