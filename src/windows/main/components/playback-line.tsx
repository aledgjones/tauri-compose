import { FC } from "react";
import { usePlayback } from "../../../services/playback-context";
import { TickConfig } from "../hooks/use-ticks";

type PlaybackLineProps = {
  ticks: TickConfig;
  height: number;
};

export const PlaybackLine: FC<PlaybackLineProps> = ({ ticks, height }) => {
  const { currentTick } = usePlayback();

  // Calculate the x position based on the current tick
  const x = currentTick * ticks.tickWidth;

  return (
    <div
      className="absolute top-0 left-0 pointer-events-none w-0.5 z-50 bg-sky-500"
      style={{
        transform: `translateX(${x}px)`,
        height: height,
      }}
    />
  );
};
