import {
  mdiBookOpenBlankVariantOutline,
  mdiDelete,
  mdiFileOutline,
  mdiPencil,
} from "@mdi/js";
import {
  AppState,
  Layout,
  LayoutType,
  SetupSelectionType,
} from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { actions } from "../../../../../services/actions";
import {
  FC,
  KeyboardEvent,
  PointerEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useAppState } from "../../../../../services/state";
import { Checkbox } from "../../../../../ui/checkbox";
import { getLayoutName } from "../../../../../services/utils";

const assignableSet = new Set<SetupSelectionType | undefined>([
  SetupSelectionType.Flow,
  SetupSelectionType.Player,
]);

const getIsAssignable = (state: AppState) => {
  return assignableSet.has(state.ui.screens.setup.selection?.selection_type);
};

const getIsAssigned = (state: AppState, layoutId: number) => {
  const selection = state.ui.screens.setup.selection;

  if (selection?.selection_type === SetupSelectionType.Player) {
    return state.score.layouts.by_id[layoutId].player_ids.includes(
      selection.id,
    );
  }

  if (selection?.selection_type === SetupSelectionType.Flow) {
    return state.score.layouts.by_id[layoutId].flow_ids.includes(selection.id);
  }

  return false;
};

type Props = {
  layout: Layout;
};

export const LayoutPanelItem: FC<Props> = ({ layout }) => {
  const inputRef = useRef<HTMLInputElement>(null);
  const savedValueRef = useRef(layout.title);
  const skipBlurCommitRef = useRef(false);

  const state = useAppState();
  const isSelected = state.ui.screens.setup.selection?.id === layout.id;
  const isAssigned = getIsAssigned(state, layout.id);
  const isAssignable = getIsAssignable(state);
  const [draftTitle, setDraftTitle] = useState(layout.title);
  const [editing, setEditing] = useState(false);

  const fg = isSelected ? "text-black" : "text-white";
  const bg = isSelected ? "bg-sky-500" : "bg-neutral-800";

  useEffect(() => {
    if (editing) {
      inputRef.current?.focus();
    }
  }, [editing]);

  const layoutName = useMemo(() => {
    if (editing) {
      return draftTitle;
    }

    return getLayoutName(state, layout.id);
  }, [draftTitle, editing, state, layout.id]);

  const onEdit = useCallback(() => {
    savedValueRef.current = layout.title;
    setDraftTitle(layout.title);
    setEditing(true);
  }, [layout.title]);

  const onDelete = useCallback(async () => {
    await actions.score.layout.remove(layout.id);
  }, [layout.id]);

  const commitRename = useCallback(async () => {
    setEditing(false);

    if (draftTitle === savedValueRef.current) {
      return;
    }

    await actions.score.layout.rename(layout.id, draftTitle);
  }, [draftTitle, layout.id]);

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

  const onAssignmentChange = useCallback(
    async (checked: boolean) => {
      const selection = state.ui.screens.setup.selection;

      if (!selection) {
        return;
      }

      if (selection.selection_type === SetupSelectionType.Flow) {
        if (checked) {
          await actions.score.layout.assignFlow(layout.id, selection.id);
        } else {
          await actions.score.layout.unassignFlow(layout.id, selection.id);
        }
      }

      if (selection.selection_type === SetupSelectionType.Player) {
        if (checked) {
          await actions.score.layout.assignPlayer(layout.id, selection.id);
        } else {
          await actions.score.layout.unassignPlayer(layout.id, selection.id);
        }
      }
    },
    [layout.id, state.ui.screens.setup.selection],
  );

  const onBlockPointerDown = useCallback(
    (e: PointerEvent<HTMLButtonElement>) => {
      e.stopPropagation();
    },
    [],
  );

  const onSelect = useCallback(() => {
    if (!isSelected) {
      actions.ui.screens.setup.select(SetupSelectionType.Layout, layout.id);
    }
  }, [isSelected, layout.id]);

  const icon = useMemo(() => {
    switch (layout.layout_type) {
      case LayoutType.Part:
        return mdiFileOutline;

      default:
        return mdiBookOpenBlankVariantOutline;
    }
  }, [layout.layout_type]);

  return (
    <div
      data-layout-id={layout.id}
      className={classNames("border-2 border-transparent", fg, bg)}
      onPointerDown={onSelect}
    >
      <div className="flex items-center h-12 text-sm pr-1 pl-3">
        <Icon className={classNames("mr-3", fg)} path={icon} size={1} />

        <input
          ref={inputRef}
          onBlur={onBlur}
          readOnly={!editing}
          value={layoutName}
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
              onChange={onAssignmentChange}
            />
          </div>
        )}
      </div>
    </div>
  );
};
