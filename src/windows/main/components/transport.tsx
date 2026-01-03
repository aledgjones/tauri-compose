import {
  mdiMetronome,
  mdiPause,
  mdiPlay,
  mdiSkipPrevious,
  mdiStop,
} from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import classNames from "classnames";
import { FC } from "react";
import { usePlayback } from "../../../services/playback-context";

type Props = {
  className?: string;
};

export const Transport: FC<Props> = ({ className }) => {
  const { isPlaying, currentTick, play, pause, stop, seek } = usePlayback();

  const handlePlayPause = () => {
    if (isPlaying) {
      pause();
    } else {
      play();
    }
  };

  const handleStop = () => {
    stop();
  };

  const handleRewind = () => {
    seek(0);
  };

  // Format tick as measure.beat.subdivision
  const formatTick = (tick: number) => {
    // This is a simplified formatting - you might want to make it more sophisticated
    const ticksPerBeat = 256; // Assuming 4/4 time with 256 ticks per quarter note
    const measure = Math.floor(tick / (ticksPerBeat * 4)) + 1;
    const beat = Math.floor((tick % (ticksPerBeat * 4)) / ticksPerBeat) + 1;
    const subdivision = tick % ticksPerBeat;
    return `${measure}.${beat}.${subdivision.toString().padStart(3, "0")}`;
  };

  return (
    <div className={classNames("flex gap-0.5", className)}>
      <IconButton
        className="text-white"
        variant="interactive"
        onClick={handleRewind}
        path={mdiSkipPrevious}
      />
      <IconButton
        variant="interactive"
        color={isPlaying ? "#ff0" : "#0f0"}
        onClick={handlePlayPause}
        path={isPlaying ? mdiPause : mdiPlay}
      />
      <IconButton
        variant="interactive"
        color="#f00"
        onClick={handleStop}
        path={mdiStop}
      />
      <p className="text-white font-mono text-sm bg-neutral-900 px-3 py-2">
        {formatTick(currentTick)}
      </p>
      <IconButton
        className="text-white"
        variant="interactive"
        onClick={() => {}}
        path={mdiMetronome}
      />
    </div>
  );
};
