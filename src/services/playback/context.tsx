import { createContext, useContext } from "react";

interface PlaybackContextType {
  isPlaying: boolean;
  currentTick: number;
  play: () => void;
  pause: () => void;
  stop: () => void;
  seek: (tick: number) => void;
}

export const PlaybackContext = createContext<PlaybackContextType | null>(null);

export function usePlayback() {
  const context = useContext(PlaybackContext);
  if (!context) {
    throw new Error("usePlayback must be used within a PlaybackProvider");
  }
  return context;
}
