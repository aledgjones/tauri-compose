import { mdiMetronome, mdiPlay, mdiRecord, mdiSkipPrevious } from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import { FunctionComponent } from "preact";
import classNames from "classnames";

type Props = {
  className?: string;
};

export const Transport: FunctionComponent<Props> = ({ className }) => {
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
