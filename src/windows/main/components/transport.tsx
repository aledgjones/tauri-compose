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

type Props = {
  className?: string;
};

export const Transport: FC<Props> = ({ className }) => {
  // TODO: Get from transport state
  const isPlaying = false;

  return (
    <div className={classNames("flex gap-0.5", className)}>
      <IconButton
        className="text-white"
        variant="interactive"
        onClick={() => {}}
        path={mdiSkipPrevious}
      />
      <IconButton
        variant="interactive"
        color={isPlaying ? "#ff0" : "#0f0"}
        onClick={() => {}}
        path={isPlaying ? mdiPause : mdiPlay}
      />
      <IconButton
        variant="interactive"
        color="#f00"
        onClick={() => {}}
        path={mdiStop}
      />
      <p className="text-white font-mono text-sm bg-neutral-900 px-3 py-2">
        1.1.000
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
