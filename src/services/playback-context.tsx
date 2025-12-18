import {
  createContext,
  useContext,
  ReactNode,
  useCallback,
  useEffect,
  useState,
} from "react";
import { transport, sampler } from "../sampler";
import { Expression } from "../sampler/types";
import { AppState, Tick, Tone } from "./types";

interface PlaybackContextType {
  isPlaying: boolean;
  currentTick: number;
  play: () => void;
  pause: () => void;
  stop: () => void;
  seek: (tick: number) => void;
}

const PlaybackContext = createContext<PlaybackContextType | null>(null);

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

    transport.clear();
    transport.subdivisions = flow.subdivisions;
    transport.length = flow.length + 1;
    transport.seek(0);

    // Schedule all events for the flow
    let totalTones = 0;

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
                totalTones++;
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

    console.log(`Scheduled ${totalTones} tones for flow`);
  }, [appState.score.flows, appState.ui.sequencer.flow]);

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

export function usePlayback() {
  const context = useContext(PlaybackContext);
  if (!context) {
    throw new Error("usePlayback must be used within a PlaybackProvider");
  }
  return context;
}
