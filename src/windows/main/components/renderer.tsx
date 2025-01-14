import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { CSSProperties } from "preact/compat";
import { useAppState } from "../../../services/state";

type Props = { className?: string; style?: CSSProperties };

export const Renderer: FunctionalComponent<Props> = ({ className, style }) => {
  const state = useAppState();

  return (
    <div
      style={{ ...style }}
      className={classNames(
        "overflow-auto bg-fixed bg-center bg-cover from-sky-700 to-sky-900 bg-gradient-to-b",
        className
      )}
    >
      <div className="p-20 flex gap-5 text-xs">
        {state.score.flows.order.map((flowId) => {
          const flow = state.score.flows.by_id[flowId];

          return (
            <pre key={flowId} className="bg-white p-5 min-w-[500px]">
              {JSON.stringify(
                {
                  ...flow,
                  player_ids: undefined,
                  master_id: undefined,
                  staves: undefined,
                  master: state.score.tracks.by_id[flow.master_id],
                  players: flow.player_ids.map((playerId) => {
                    const player = state.score.players.by_id[playerId];

                    return {
                      ...player,
                      instruments: player.instrument_ids.map((instrumentId) => {
                        const instrument =
                          state.score.instruments.by_id[instrumentId];
                        return {
                          ...instrument,
                          staves: instrument.stave_ids.map((staveId) => {
                            const stave = flow.staves.by_id[staveId];
                            return {
                              ...stave,
                              master: state.score.tracks.by_id[stave.master_id],
                              tracks: stave.track_ids.map((trackId) => {
                                const track = state.score.tracks.by_id[trackId];

                                return {
                                  ...track,
                                };
                              }),
                            };
                          }),
                        };
                      }),
                    };
                  }),
                },
                (key, value) => {
                  if (
                    Array.isArray(value) &&
                    value.every((item) => typeof item === "number")
                  ) {
                    return JSON.stringify(value); // Stringify int arrays on one line
                  } else {
                    return value; // Keep other values as is
                  }
                },
                2
              )}
            </pre>
          );
        })}
      </div>
    </div>
  );
};
