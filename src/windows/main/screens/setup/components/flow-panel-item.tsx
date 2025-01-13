import { mdiDelete, mdiHeatWave, mdiPencil } from "@mdi/js";
import { Flow, SelectionType } from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { IconButton } from "../../../../../ui/icon-button";
import { actions } from "../../../../../services/actions";
import { Checkbox } from "../../../../../ui/checkbox";

type Props = {
  flow: Flow;
  isSelected: boolean;
  isAssignable: boolean;
  isAssigned: boolean;
};

export const FlowPanelItem: FunctionalComponent<Props> = ({
  flow,
  isSelected,
  isAssignable,
  isAssigned,
}) => {
  const fg = isSelected ? "text-black" : "text-white";
  const bg = isSelected ? "bg-sky-500" : "bg-neutral-800";

  return (
    <div
      data-flow-id={flow.id}
      className={classNames(
        "flex-1 border-2 border-transparent min-w-[260px]",
        fg,
        bg
      )}
      onPointerDown={() => {
        if (!isSelected) {
          actions.ui.setup.select(SelectionType.Flow, flow.id);
        }
      }}
    >
      <div
        className={classNames("text-sm", {
          "pl-3 pr-2": isAssignable,
          "px-3": !isAssignable,
        })}
      >
        <div className="flex items-center h-[48px]">
          <Icon
            className={classNames("mr-3", fg)}
            path={mdiHeatWave}
            size={1}
          />

          <input
            readOnly
            value={flow.title || "Untitled Flow"}
            className={classNames(`truncate flex-1 bg-transparent`, fg)}
          />

          {isSelected && (
            <>
              <IconButton
                className={fg}
                variant="standard"
                path={mdiPencil}
                onClick={async () => {
                  // edit tilte
                }}
              />
              <IconButton
                className={fg}
                variant="standard"
                path={mdiDelete}
                onClick={async () => {
                  // delete
                }}
              />
            </>
          )}

          {isAssignable && (
            <div className="mr-[-2px]">
              <Checkbox
                value={isAssigned}
                onPointerDown={(e) => e.stopPropagation()}
                onChange={() => {}}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
