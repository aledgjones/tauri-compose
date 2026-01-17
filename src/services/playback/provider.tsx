import { ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { AppState, Tick, Tone } from "../types";
import { Expression, sampler, transport } from "../../sampler";
import { PlaybackContext } from "./context";

interface PlaybackProviderProps {
  appState: AppState;
  children: ReactNode;
}

export function PlaybackProvider({
  appState,
  children,
}: PlaybackProviderProps) {
  const [isPlaying, setIsPlaying] = useState(false);
  const [currentTick, setCurrentTick] = useState(transport.tick);

  // Track scheduling-relevant state to avoid re-scheduling on volume changes
  const lastSchedulingStateRef = useRef<string>("");

  const play = useCallback(async () => {
    if (isPlaying) return;

    if (transport.ctx.state === "suspended") {
      await transport.ctx.resume();
    }

    transport.start();
  }, [isPlaying]);

  const pause = useCallback(() => {
    transport.pause();
    sampler.stopAll();
  }, []);

  const stop = useCallback(() => {
    transport.stop();
    sampler.stopAll();
  }, []);

  const seek = useCallback((tick: Tick) => {
    transport.seek(tick);
    setCurrentTick(tick);
  }, []);

  // Initialize transport when flow changes
  useEffect(() => {
    const flowId = appState.ui.sequencer.flow || appState.score.flows.order[0];
    if (!flowId) return;

    const flow = appState.score.flows.by_id[flowId];
    if (!flow) return;

    // Create a stable key representing scheduling-relevant state
    const schedulingKey = JSON.stringify({
      flowId,
      flow: {
        length: flow.length,
        player_ids: flow.player_ids,
        staves: flow.staves,
      },
      players: appState.score.players,
      tracks: appState.score.tracks,
      instruments: appState.score.instruments,
    });

    // Only re-schedule if scheduling-relevant data has changed
    if (schedulingKey === lastSchedulingStateRef.current) {
      return;
    }
    lastSchedulingStateRef.current = schedulingKey;

    transport.clear();
    transport.subdivisions = 32;
    transport.length = flow.length + 1;
    transport.seek(0);

    // Schedule tones from all tracks in this flow
    flow.player_ids.forEach((playerId) => {
      const player = appState.score.players.by_id[playerId];
      if (!player) return;

      player.instrument_ids.forEach((instrumentId) => {
        const instrument = appState.score.instruments.by_id[instrumentId];
        if (!instrument) return;

        // Find staves for this instrument
        instrument.stave_ids.forEach((staveId) => {
          const stave = flow.staves.by_id[staveId];
          if (!stave) return;

          // Find tracks for this stave
          stave.track_ids.forEach((trackId) => {
            const track = appState.score.tracks.by_id[trackId];
            if (!track) return;

            // Schedule all tones in this track
            Object.values(track.entries.by_id)
              .filter((entry): entry is Tone => entry.type === "Tone")
              .forEach((tone) => {
                transport.scheduleEvent(
                  tone.tick,
                  tone.duration,
                  (when: number, duration: number) => {
                    sampler.play(
                      instrumentId.toString(),
                      Expression.Natural,
                      tone.pitch.int,
                      when,
                      duration,
                    );
                  },
                );
              });
          });
        });
      });
    });
  }, [
    appState.score.flows,
    appState.score.instruments,
    appState.score.players,
    appState.score.tracks,
    appState.ui.sequencer.flow,
  ]);

  useEffect(() => {
    const onStart = () => {
      setIsPlaying(true);
    };

    const onStop = () => {
      setIsPlaying(false);
    };

    const onTick = (tick: Tick) => {
      setCurrentTick(tick);
    };

    transport.on("start", onStart);
    transport.on("pause", onStop);
    transport.on("stop", onStop);
    transport.on("tick", onTick);

    return () => {
      transport.removeListener("start", onStart);
      transport.removeListener("pause", onStop);
      transport.removeListener("stop", onStop);
      transport.removeListener("tick", onTick);
    };
  }, []);

  const value = {
    isPlaying,
    currentTick,
    play,
    pause,
    stop,
    seek,
  };

  return (
    <PlaybackContext.Provider value={value}>
      {children}
    </PlaybackContext.Provider>
  );
}
