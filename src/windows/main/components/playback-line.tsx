import { FC } from "react";
import { TickConfig } from "../hooks/use-ticks";
import { useTransportState } from "../../../services/state/context";

type PlaybackLineProps = {
  ticks: TickConfig;
  height: number;
};

export const PlaybackLine: FC<PlaybackLineProps> = ({ ticks, height }) => {
  const transport = useTransportState();
  const currentTick = transport.current_tick;

  // Calculate the x position based on the current tick
  const x = currentTick * ticks.tickWidth;

  return (
    <div
      className="absolute top-0 left-0 pointer-events-none w-0.5 z-20 bg-sky-500"
      style={{
        transform: `translateX(${x}px)`,
        height: height,
      }}
    />
  );
};
