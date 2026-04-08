import { mdiDelete, mdiHeatWave, mdiPencil } from "@mdi/js";
import {
  AppState,
  Flow,
  SetupSelectionType,
} from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { actions } from "../../../../../services/actions";
import { Checkbox } from "../../../../../ui/checkbox";
import { useAppState } from "../../../../../services/state";
import {
  FC,
  KeyboardEvent,
  PointerEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

const assignableSet = new Set<SetupSelectionType | undefined>([
  SetupSelectionType.Player,
  SetupSelectionType.Layout,
]);

const getIsAssigned = (state: AppState, flowId: number) => {
  const selection = state.ui.screens.setup.selection;

  if (selection?.selection_type === SetupSelectionType.Player) {
    return state.score.flows.by_id[flowId].player_ids.includes(selection.id);
  }

  if (selection?.selection_type === SetupSelectionType.Layout) {
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
  const savedValueRef = useRef(flow.title);
  const skipBlurCommitRef = useRef(false);

  const state = useAppState();
  const [draftTitle, setDraftTitle] = useState(flow.title);
  const [editing, setEditing] = useState(false);

  const isSelected = state.ui.screens.setup.selection?.id === flow.id;
  const isAssigned = getIsAssigned(state, flow.id);
  const isAssignable = assignableSet.has(
    state.ui.screens.setup.selection?.selection_type,
  );

  const fg = isSelected ? "text-black" : "text-white";
  const bg = isSelected ? "bg-sky-500" : "bg-neutral-800";

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus();
    }
  }, [editing]);

  const title = editing ? draftTitle : flow.title || "Untitled Flow";

  const onEdit = useCallback(() => {
    savedValueRef.current = flow.title;
    setDraftTitle(flow.title);
    setEditing(true);
  }, [flow.title]);

  const onDelete = useCallback(async () => {
    await actions.score.flow.remove(flow.id);
  }, [flow.id]);

  const commitRename = useCallback(async () => {
    setEditing(false);

    if (draftTitle === savedValueRef.current) {
      return;
    }

    await actions.score.flow.rename(flow.id, draftTitle);
  }, [draftTitle, flow.id]);

  const onKeyDown = useCallback(
    async (e: KeyboardEvent<HTMLInputElement>) => {
      switch (e.key) {
        case "Enter":
          await commitRename();
          inputRef.current?.blur();
          break;
        case "Escape":
          skipBlurCommitRef.current = true;
          setDraftTitle(savedValueRef.current);
          setEditing(false);
          inputRef.current?.blur();
          break;
        default:
          break;
      }
    },
    [commitRename],
  );

  const onBlur = useCallback(async () => {
    if (skipBlurCommitRef.current) {
      skipBlurCommitRef.current = false;
      return;
    }

    await commitRename();
  }, [commitRename]);

  const onToggleAssigned = useCallback(
    async (checked: boolean) => {
      if (!state.ui.screens.setup.selection) {
        return;
      }

      if (
        state.ui.screens.setup.selection.selection_type ===
        SetupSelectionType.Player
      ) {
        if (checked) {
          await actions.score.flow.assignPlayer(
            flow.id,
            state.ui.screens.setup.selection.id,
          );
        } else {
          await actions.score.flow.unassignPlayer(
            flow.id,
            state.ui.screens.setup.selection.id,
          );
        }
      }

      if (
        state.ui.screens.setup.selection.selection_type ===
        SetupSelectionType.Layout
      ) {
        if (checked) {
          await actions.score.layout.assignFlow(
            state.ui.screens.setup.selection.id,
            flow.id,
          );
        } else {
          await actions.score.layout.unassignFlow(
            state.ui.screens.setup.selection.id,
            flow.id,
          );
        }
      }
    },
    [state.ui.screens.setup.selection, flow.id],
  );

  const onSelect = useCallback(() => {
    if (!isSelected) {
      actions.ui.screens.setup.select(SetupSelectionType.Flow, flow.id);
    }
  }, [flow.id, isSelected]);

  const onBlockPointerDown = useCallback(
    (e: PointerEvent<HTMLButtonElement>) => {
      e.stopPropagation();
    },
    [],
  );

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
            onChange={(e) => setDraftTitle(e.currentTarget.value)}
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
