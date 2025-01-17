import {
  mdiBookOpenBlankVariantOutline,
  mdiChevronDown,
  mdiDelete,
  mdiPencil,
} from "@mdi/js";
import {
  AppState,
  Layout,
  LayoutType,
  SelectionType,
} from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { IconButton } from "../../../../../ui/icon-button";
import { actions } from "../../../../../services/actions";
import { useCallback, useMemo } from "preact/hooks";
import { useAppState } from "../../../../../services/state";
import { Checkbox } from "../../../../../ui/checkbox";

const assignableSet = new Set<SelectionType | undefined>([
  SelectionType.Flow,
  SelectionType.Player,
]);

const getIsAssignable = (state: AppState) => {
  return assignableSet.has(state.ui.setup.selection?.selection_type);
};

const getIsAssigned = (state: AppState, layoutId: number) => {
  const selection = state.ui.setup.selection;

  if (selection?.selection_type === SelectionType.Player) {
    return state.score.layouts.by_id[layoutId].player_ids.includes(
      selection.id
    );
  }

  if (selection?.selection_type === SelectionType.Flow) {
    return state.score.layouts.by_id[layoutId].flow_ids.includes(selection.id);
  }

  return false;
};

type Props = {
  layout: Layout;
};

export const LayoutPanelItem: FunctionalComponent<Props> = ({ layout }) => {
  const state = useAppState();

  const isExpanded = state.ui.setup.expanded.includes(layout.id);
  const isSelected = state.ui.setup.selection?.id === layout.id;
  const isAssigned = getIsAssigned(state, layout.id);
  const isAssignable = getIsAssignable(state);

  const fg = isSelected ? "text-black" : "text-white";
  const bg = isSelected ? "bg-sky-500" : "bg-neutral-800";

  const layoutName = useMemo(() => {
    if (layout.title) {
      return layout.title;
    }

    if (layout.layout_type === LayoutType.FullScore) {
      return "Full Score";
    }

    const instrumentNames = state.score.players.order
      .filter((playerId) => layout.player_ids.includes(playerId))
      .flatMap((playerId) => {
        const player = state.score.players.by_id[playerId];
        return player.instrument_ids.map((instrumentId) => {
          const instrument = state.score.instruments.by_id[instrumentId];
          return (
            instrument.long_name +
            (instrument.count ? ` ${instrument.count}` : "")
          );
        });
      });

    if (instrumentNames.length === 0) {
      return "Empty Part";
    }

    if (instrumentNames.length === 1) {
      return instrumentNames[0];
    }

    const last = instrumentNames.pop();
    return `${instrumentNames.join(", ")} & ${last}`;
  }, [
    layout.layout_type,
    layout.player_ids,
    layout.title,
    state.score.instruments.by_id,
    state.score.players.by_id,
    state.score.players.order,
  ]);

  const onAssignmentChange = useCallback(
    async (checked: boolean) => {
      const selection = state.ui.setup.selection;

      if (!selection) {
        return;
      }

      if (selection.selection_type === SelectionType.Flow) {
        if (checked) {
          await actions.score.layout.assignFlow(layout.id, selection.id);
        } else {
          await actions.score.layout.unassignFlow(layout.id, selection.id);
        }
      }

      if (selection.selection_type === SelectionType.Player) {
        if (checked) {
          await actions.score.layout.assignPlayer(layout.id, selection.id);
        } else {
          await actions.score.layout.unassignPlayer(layout.id, selection.id);
        }
      }
    },
    [layout.id, state.ui.setup.selection]
  );

  return (
    <div
      data-layout-id={layout.id}
      className={classNames("border-2 border-transparent", fg, bg)}
      onPointerDown={() => {
        if (!isSelected) {
          actions.ui.setup.select(SelectionType.Layout, layout.id);
        }
      }}
    >
      <div className="flex items-center h-[48px] text-sm pr-1 pl-3">
        <Icon
          className={classNames("mr-3", fg)}
          path={mdiBookOpenBlankVariantOutline}
          size={1}
        />

        <p className={classNames(`truncate flex-1`, fg)}>{layoutName}</p>

        {isSelected && (
          <>
            <IconButton
              className={fg}
              variant="standard"
              path={mdiPencil}
              onClick={() => {}}
            />
            <IconButton
              className={fg}
              variant="standard"
              path={mdiDelete}
              onClick={() => {}}
            />
          </>
        )}

        {isAssignable && (
          <div className="mr-[-2px]">
            <Checkbox
              value={isAssigned}
              onPointerDown={(e) => e.stopPropagation()}
              onChange={onAssignmentChange}
            />
          </div>
        )}

        {!isAssignable && (
          <div onPointerDown={(e) => e.stopPropagation()}>
            <IconButton
              variant="standard"
              path={mdiChevronDown}
              className={classNames("transition-transform", fg, {
                "rotate-180": isExpanded,
              })}
              onClick={async () => {
                if (isExpanded) {
                  await actions.ui.setup.collapse(layout.id);
                } else {
                  await actions.ui.setup.expand(layout.id);
                }
              }}
            />
          </div>
        )}
      </div>
      <div
        className={classNames("flex flex-col p-2 gap-2 min-h-[64px]", {
          hidden: !isExpanded,
        })}
      >
        Content
      </div>
    </div>
  );
};
