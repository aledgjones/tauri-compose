import { mdiDelete, mdiHeatWave, mdiPencil } from "@mdi/js";
import { AppState, Flow, SelectionType } from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { IconButton } from "../../../../../ui/icon-button";
import { actions } from "../../../../../services/actions";
import { Checkbox } from "../../../../../ui/checkbox";
import { useAppState } from "../../../../../services/state";
import { FC, PointerEvent, useCallback, useRef, useState } from "react";

const assignableSet = new Set<SelectionType | undefined>([
  SelectionType.Player,
  SelectionType.Layout,
]);

const getIsAssigned = (state: AppState, flowId: number) => {
  const selection = state.ui.setup.selection;

  if (selection?.selection_type === SelectionType.Player) {
    return state.score.flows.by_id[flowId].player_ids.includes(selection.id);
  }

  if (selection?.selection_type === SelectionType.Layout) {
    return state.score.layouts.by_id[selection.id].flow_ids.includes(flowId);
  }

  return false;
};

type Props = {
  index: number;
  flow: Flow;
};

export const FlowPanelItem: FC<Props> = ({ flow, index }) => {
  const inputRef = useRef<HTMLInputElement>(null);

  const state = useAppState();
  const [savedValue, setSavedValue] = useState(flow.title);
  const [editing, setEditing] = useState(false);

  const isSelected = state.ui.setup.selection?.id === flow.id;
  const isAssigned = getIsAssigned(state, flow.id);
  const isAssignable = assignableSet.has(
    state.ui.setup.selection?.selection_type,
  );

  const fg = isSelected ? "text-black" : "text-white";
  const bg = isSelected ? "bg-sky-500" : "bg-neutral-800";

  const title = editing ? flow.title : flow.title || "Untitled Flow";

  const onEdit = useCallback(() => {
    if (inputRef.current) {
      setSavedValue(flow.title);
      inputRef.current.focus();
    }
    setEditing(true);
  }, [flow.title]);

  const onDelete = useCallback(async () => {
    await actions.score.flow.remove(flow.id);
  }, [flow.id]);

  const onKeyDown = useCallback(
    async (e: any) => {
      switch (e.key) {
        case "Enter":
          // confirm
          if (inputRef.current) {
            inputRef.current.blur();
          }
          break;
        case "Escape":
          // revert
          await actions.score.flow.rename(flow.id, savedValue);
          inputRef.current?.blur();
          break;
        default:
          break;
      }
    },
    [flow.id, savedValue],
  );

  const onInput = useCallback(
    async (e: any) => {
      await actions.score.flow.rename(flow.id, e.target.value);
    },
    [flow.id],
  );

  const onBlur = useCallback(() => {
    setEditing(false);
  }, []);

  const onToggleAssigned = useCallback(
    async (checked: boolean) => {
      if (!state.ui.setup.selection) {
        return;
      }

      if (state.ui.setup.selection.selection_type === SelectionType.Player) {
        if (checked) {
          await actions.score.flow.assignPlayer(
            flow.id,
            state.ui.setup.selection.id,
          );
        } else {
          await actions.score.flow.unassignPlayer(
            flow.id,
            state.ui.setup.selection.id,
          );
        }
      }

      if (state.ui.setup.selection.selection_type === SelectionType.Layout) {
        if (checked) {
          await actions.score.layout.assignFlow(
            state.ui.setup.selection.id,
            flow.id,
          );
        } else {
          await actions.score.layout.unassignFlow(
            state.ui.setup.selection.id,
            flow.id,
          );
        }
      }
    },
    [state.ui.setup.selection, flow.id],
  );

  const onSelect = useCallback(() => {
    if (!isSelected) {
      actions.ui.setup.select(SelectionType.Flow, flow.id);
    }
  }, [flow.id, isSelected]);

  const onBlockPointerDown = useCallback((e: PointerEvent<HTMLDivElement>) => {
    e.stopPropagation();
  }, []);

  return (
    <div
      data-flow-id={flow.id}
      className={classNames(
        "flex flex-col flex-1 border-2 border-transparent min-w-65",
        fg,
        bg,
      )}
      onPointerDown={onSelect}
    >
      <div
        className={classNames("text-sm", {
          "pl-3 pr-2": isAssignable,
          "px-2": !isAssignable,
        })}
      >
        <div className="flex items-center h-12">
          <Icon
            className={classNames("mr-3", fg)}
            path={mdiHeatWave}
            size={1}
          />

          <input
            ref={inputRef}
            onBlur={onBlur}
            readOnly={!editing}
            value={title}
            className={classNames(
              `truncate flex-1 bg-transparent outline-hidden`,
              fg,
            )}
            onKeyDown={onKeyDown}
            onInput={onInput}
          />

          {isSelected && (
            <>
              <IconButton
                className={fg}
                variant="standard"
                path={mdiPencil}
                onClick={onEdit}
              />
              <IconButton
                className={fg}
                variant="standard"
                path={mdiDelete}
                onClick={onDelete}
              />
            </>
          )}

          {isAssignable && (
            <div className="-mr-0.5">
              <Checkbox
                value={isAssigned}
                onPointerDown={onBlockPointerDown}
                onChange={onToggleAssigned}
              />
            </div>
          )}
        </div>
      </div>
      <div className="flex items-end justify-end flex-1 px-2">
        <div className="flex items-center justify-center h-12 w-9">
          {index + 1}
        </div>
      </div>
    </div>
  );
};
