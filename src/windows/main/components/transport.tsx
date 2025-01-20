import { mdiMetronome, mdiPlay, mdiRecord, mdiSkipPrevious } from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import classNames from "classnames";
import { FC } from "react";

type Props = {
  className?: string;
};

export const Transport: FC<Props> = ({ className }) => {
  return (
    <div className={classNames("flex gap-1", className)}>
      <IconButton
        className="text-white"
        variant="filled"
        onClick={() => {}}
        path={mdiSkipPrevious}
      />
      <IconButton
        variant="filled"
        color="#0f0"
        onClick={() => {}}
        path={mdiPlay}
      />
      <IconButton
        variant="filled"
        color="#f00"
        onClick={() => {}}
        path={mdiRecord}
      />
      <p className="text-white font-mono text-sm bg-neutral-900 px-3 py-2">
        1.1.1.000
      </p>
      <IconButton
        className="text-white"
        variant="filled"
        onClick={() => {}}
        path={mdiMetronome}
      />
    </div>
  );
};
