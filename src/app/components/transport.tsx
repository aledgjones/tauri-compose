import {
  mdiMetronome,
  mdiMovieRoll,
  mdiPlay,
  mdiRecord,
  mdiSkipPrevious,
} from "@mdi/js";
import { IconButton } from "../../ui/icon-button";

export const Transport = () => {
  return (
    <div className="flex">
      <IconButton
        className="mr-1"
        onClick={() => {}}
        variant="filled"
        path={mdiSkipPrevious}
      />
      <IconButton
        className="mr-1"
        color="#0f0"
        onClick={() => {}}
        variant="filled"
        path={mdiPlay}
      />
      <IconButton
        className="mr-1"
        color="#f00"
        onClick={() => {}}
        variant="filled"
        path={mdiRecord}
      />
      <p className="text-white font-mono text-sm bg-neutral-800 px-3 py-2 mr-1 rounded-lg">
        1.1.1.000
      </p>
      <IconButton
        className="mr-1"
        onClick={() => {}}
        variant="filled"
        path={mdiMetronome}
      />
      <IconButton onClick={() => {}} variant="filled" path={mdiMovieRoll} />
    </div>
  );
};
