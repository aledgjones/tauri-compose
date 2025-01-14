import { mdiDelete, mdiHeatWave, mdiPencil } from "@mdi/js";
import {
  AppState,
  Flow,
  SelectionType,
  UniqueId,
} from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { IconButton } from "../../../../../ui/icon-button";
import { actions } from "../../../../../services/actions";
import { Checkbox } from "../../../../../ui/checkbox";
import { useAppState } from "../../../../../services/state";

const assignableSet = new Set<SelectionType | undefined>([
  SelectionType.Player,
  SelectionType.Layout,
]);

const getIsAssigned = (state: AppState, flowId: number) => {
  const selection = state.ui.setup.selection;

  if (selection?.selection_type === SelectionType.Player) {
    return state.score.flows.by_id[flowId].player_ids.includes(selection.id);
  }

  return false;
};

type Props = {
  flow: Flow;
};

export const FlowPanelItem: FunctionalComponent<Props> = ({ flow }) => {
  const state = useAppState();
  const selection = state.ui.setup.selection;

  const isSelected = selection?.id === flow.id;
  const isAssigned = getIsAssigned(state, flow.id);
  const isAssignable = assignableSet.has(selection?.selection_type);

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
                onChange={async (checked) => {
                  if (!selection) {
                    return;
                  }

                  if (selection.selection_type === SelectionType.Player) {
                    if (checked) {
                      await actions.score.flow.assignPlayer(
                        flow.id,
                        selection.id
                      );
                    } else {
                      await actions.score.flow.unassignPlayer(
                        flow.id,
                        selection.id
                      );
                    }
                  }
                }}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
