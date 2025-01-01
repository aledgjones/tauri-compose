import { mdiFolderCog, mdiRedo, mdiUndo } from "@mdi/js";
import { IconButton } from "../../ui/icon-button";
import { FunctionComponent } from "preact";
import classNames from "classnames";

type Props = {
  className?: string;
};

export const Tools: FunctionComponent<Props> = ({ className }) => {
  return (
    <div className={classNames("flex", className)}>
      <IconButton
        className="mr-2"
        onClick={() => {}}
        variant="filled"
        path={mdiFolderCog}
      />
      <IconButton
        className="mr-1"
        onClick={() => {}}
        variant="filled"
        path={mdiUndo}
      />
      <IconButton onClick={() => {}} variant="filled" path={mdiRedo} />
    </div>
  );
};
