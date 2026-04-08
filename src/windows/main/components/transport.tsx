import { mdiPause, mdiPlay, mdiSkipPrevious } from "@mdi/js";
import { IconButton } from "../../../ui/buttons/icon-button";
import classNames from "classnames";
import { FC, useMemo } from "react";
import { useTransportState } from "../../../services/state/context";
import { actions } from "../../../services/actions";

type Props = {
  className?: string;
};

const TICKS_PER_QUARTER = 32;

/**
 * Format a tick position as bar.beat.subdivision
 * Assumes 4/4 time for now.
 */
function formatPosition(tick: number): string {
  const ticksPerBeat = TICKS_PER_QUARTER;
  const beatsPerBar = 4;
  const ticksPerBar = ticksPerBeat * beatsPerBar;

  const bar = Math.floor(tick / ticksPerBar) + 1;
  const beatInBar = Math.floor((tick % ticksPerBar) / ticksPerBeat) + 1;
  const subInBeat = tick % ticksPerBeat;

  // Format subdivision as 3-digit number (0-31 range)
  const subStr = String(subInBeat).padStart(3, "0");

  return `${bar}.${beatInBar}.${subStr}`;
}

export const Transport: FC<Props> = ({ className }) => {
  const transport = useTransportState();
  const isPlaying = transport.playing;

  const position = useMemo(
    () => formatPosition(transport.current_tick),
    [transport.current_tick],
  );

  const handlePlayPause = () => {
    if (isPlaying) {
      actions.transport.pause();
    } else {
      actions.transport.play();
    }
  };

  const handleSkipPrevious = () => {
    actions.transport.seek(0);
  };

  return (
    <div className={classNames("flex gap-0.5", className)}>
      <IconButton
        className="text-white"
        variant="interactive"
        onClick={handleSkipPrevious}
        path={mdiSkipPrevious}
        aria-label="Return transport to start"
        title="Return transport to start"
      />
      <IconButton
        variant="interactive"
        color={isPlaying ? "#ff0" : "#0f0"}
        onClick={handlePlayPause}
        path={isPlaying ? mdiPause : mdiPlay}
        aria-label={isPlaying ? "Pause playback" : "Start playback"}
        title={isPlaying ? "Pause playback" : "Start playback"}
      />
      <p className="text-white font-mono text-sm bg-neutral-900 px-3 py-2">
        {position}
      </p>
      <p className="text-white font-mono text-sm bg-neutral-900 px-3 py-2 min-w-20 text-center">
        {`${Math.round(transport.tempo)} BPM`}
      </p>
    </div>
  );
};
