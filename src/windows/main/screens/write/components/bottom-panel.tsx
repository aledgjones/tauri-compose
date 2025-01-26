import { FC, useMemo } from "react";
import { useAppState } from "../../../../../services/state";
import { Select, SelectOption } from "../../../../../ui/select";
import { UniqueId } from "../../../../../services/types";
import { actions } from "../../../../../services/actions";

const ALPHABET = ["A", "B"];
const TRACK_LABEL = ["Up-stem", "Down-stem"];

export const BottomPanel: FC = () => {
  const state = useAppState();

  const instruments: SelectOption<UniqueId>[] = useMemo(() => {
    return state.score.players.order.flatMap((playerId) => {
      const player = state.score.players.by_id[playerId];
      return player.instrument_ids.map((instrumentId) => {
        const instrument = state.score.instruments.by_id[instrumentId];
        return {
          value: instrumentId,
          label:
            instrument.long_name +
            (instrument.count ? ` ${instrument.count}` : ""),
        };
      });
    });
  }, [state]);

  const flowId = useMemo(() => {
    return state.score.flows.order[0] || null;
  }, [state]);

  const instrumentId = useMemo(() => {
    if (instruments.length == 0) {
      return null;
    }

    const id = state.ui.sequencer.instrument;

    // check the selection still exists
    if (
      id &&
      instruments.some((option: SelectOption<UniqueId>) => option.value === id)
    ) {
      return id;
    }

    // fallback to the first instrument
    return instruments[0].value;
  }, [state]);

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

    return stave_ids.flatMap((staveId, i) => {
      const stave = flow.staves.by_id[staveId];

      if (!stave) {
        return [];
      }

      return stave.track_ids.map((trackId, ii) => {
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
  }, [state]);

  const trackId = useMemo(() => {
    if (tracks.length == 0) {
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
  }, [state]);

  console.log(tracks);

  return (
    <div className="bg-neutral-900 flex flex-col">
      <div className="flex p-2 h-[52px] items-center">
        {instrumentId && instruments.length > 0 && (
          <Select
            variant="standard"
            value={instrumentId}
            options={instruments}
            onChange={actions.ui.sequencer.setInstrument}
          />
        )}
        {trackId && tracks.length > 0 && (
          <Select
            variant="standard"
            value={trackId}
            options={tracks}
            onChange={actions.ui.sequencer.setTrack}
          />
        )}
      </div>
      {!flowId && (
        <div className="flex flex-1 items-center justify-center">{/*  */}</div>
      )}
    </div>
  );
};
