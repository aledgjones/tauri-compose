import {
  mdiAccount,
  mdiAccountGroup,
  mdiChevronDown,
  mdiDelete,
  mdiPlus,
} from "@mdi/js";
import {
  AppState,
  Player,
  PlayerType,
  SelectionType,
} from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { IconButton } from "../../../../../ui/icon-button";
import { actions } from "../../../../../services/actions";
import {
  FC,
  PointerEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
} from "react";
import { PlayerPanelInstrumentListItem } from "./player-panel-instrument-list-item";
import Sortable from "sortablejs";
import { useAppState } from "../../../../../services/state";
import { Checkbox } from "../../../../../ui/checkbox";
import { TokenizedText } from "../../../../../ui/text";

const assignableSet = new Set<SelectionType | undefined>([
  SelectionType.Flow,
  SelectionType.Layout,
]);

const getIsAssignable = (state: AppState) => {
  return assignableSet.has(state.ui.screens.setup.selection?.selection_type);
};

const getIsAssigned = (state: AppState, playerId: number) => {
  const selection = state.ui.screens.setup.selection;

  if (selection?.selection_type === SelectionType.Flow) {
    return state.score.flows.by_id[selection.id].player_ids.includes(playerId);
  }

  if (selection?.selection_type === SelectionType.Layout) {
    return state.score.layouts.by_id[selection.id].player_ids.includes(
      playerId,
    );
  }

  return false;
};

type Props = {
  player: Player;
};

export const PlayerPanelItem: FC<Props> = ({ player }) => {
  const state = useAppState();

  const isExpanded = state.ui.screens.setup.expanded.includes(player.id);
  const isSelected = state.ui.screens.setup.selection?.id === player.id;
  const isAssigned = getIsAssigned(state, player.id);
  const isAssignable = getIsAssignable(state);
  const isInstrumentAssignable =
    player.player_type === PlayerType.Solo ||
    player.instrument_ids.length === 0;

  const ref = useRef<HTMLDivElement | null>(null);

  const fg = isSelected ? "text-black" : "text-white";
  const bg = isSelected ? "bg-sky-500" : "bg-neutral-800";

  const icon = useMemo(() => {
    switch (player.player_type) {
      case PlayerType.Solo:
        return mdiAccount;
      default:
        return mdiAccountGroup;
    }
  }, [player.player_type]);

  const playerName = useMemo(() => {
    const instrumentNames = player.instrument_ids.map((instrumentId) => {
      const instrument = state.score.instruments.by_id[instrumentId];
      return (
        instrument.long_name + (instrument.count ? ` ${instrument.count}` : "")
      );
    });

    if (instrumentNames.length === 0) {
      return "Empty-handed Player";
    }

    if (instrumentNames.length === 1) {
      return instrumentNames[0];
    }

    const last = instrumentNames.pop();

    return `${instrumentNames.join(", ")} & ${last}`;
  }, [player, state.score.instruments.by_id]);

  const onAssignmentChange = useCallback(
    async (checked: boolean) => {
      const selection = state.ui.screens.setup.selection;

      if (!selection) {
        return;
      }

      if (selection.selection_type === SelectionType.Flow) {
        if (checked) {
          await actions.score.flow.assignPlayer(selection.id, player.id);
        } else {
          await actions.score.flow.unassignPlayer(selection.id, player.id);
        }
      }

      if (selection.selection_type === SelectionType.Layout) {
        if (checked) {
          await actions.score.layout.assignPlayer(selection.id, player.id);
        } else {
          await actions.score.layout.unassignPlayer(selection.id, player.id);
        }
      }
    },
    [player.id, state.ui.screens.setup.selection],
  );

  useEffect(() => {
    if (!ref.current) {
      return;
    }

    const sortable = Sortable.create(ref.current, {
      animation: 150,
      forceFallback: true,
      ghostClass: "ghost-instrument",
      delay: 150,
      group: "instruments",
      onSort: async (e) => {
        if (
          e.oldIndex === undefined ||
          e.newIndex === undefined ||
          !e.from.dataset.playerId ||
          !e.to.dataset.playerId
        ) {
          return;
        }

        // only handle the sort once in the destination player (can be itself)
        if (parseInt(e.to.dataset.playerId) === player.id) {
          await actions.score.instrument.move(
            e.oldIndex,
            e.newIndex,
            parseInt(e.from.dataset.playerId),
            parseInt(e.to.dataset.playerId),
          );
          await actions.ui.screens.setup.select(SelectionType.Player, player.id);
        }
      },
    });

    return () => {
      sortable.destroy();
    };
  }, [player.id]);

  const onDelete = useCallback(async () => {
    await actions.score.player.remove(player.id);
  }, [player.id]);

  const onAddInstrument = useCallback(async () => {
    const definitionId = await actions.pickAnInstrument();
    if (definitionId) {
      const instrumentId = await actions.score.instrument.create(definitionId);
      await actions.score.player.assignInstrument(player.id, instrumentId);
    }
  }, [player.id]);

  const onToggleExpand = useCallback(async () => {
    if (isExpanded) {
      await actions.ui.screens.setup.collapse(player.id);
    } else {
      await actions.ui.screens.setup.expand(player.id);
    }
  }, [isExpanded, player.id]);

  const onBlockPointerDown = useCallback((e: PointerEvent<HTMLDivElement>) => {
    e.stopPropagation();
  }, []);

  const onSelect = useCallback(() => {
    if (!isSelected) {
      actions.ui.screens.setup.select(SelectionType.Player, player.id);
    }
  }, [isSelected, player.id]);

  return (
    <div
      data-player-id={player.id}
      className={classNames("border-2 border-transparent", fg, bg)}
      onPointerDown={onSelect}
    >
      <div className="flex items-center h-12 text-sm pr-1 pl-3">
        <Icon className={classNames("mr-3", fg)} path={icon} size={1} />

        <p className={classNames(`truncate flex-1`, fg)}>
          <TokenizedText>{playerName}</TokenizedText>
        </p>

        {isSelected && (
          <>
            <IconButton
              className={fg}
              variant="standard"
              path={mdiDelete}
              onClick={onDelete}
            />
            {isInstrumentAssignable && (
              <IconButton
                className={fg}
                variant="standard"
                path={mdiPlus}
                onClick={onAddInstrument}
              />
            )}
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

        {!isAssignable && (
          <div onPointerDown={onBlockPointerDown}>
            <IconButton
              variant="standard"
              path={mdiChevronDown}
              className={classNames("transition-transform", fg, {
                "rotate-180": isExpanded,
              })}
              onClick={onToggleExpand}
            />
          </div>
        )}
      </div>
      <div
        ref={ref}
        data-player-id={player.id}
        className={classNames("flex flex-col p-2 gap-2 min-h-16", {
          hidden: !isExpanded,
        })}
      >
        {player.instrument_ids.map((instrumentId) => {
          const instrument = state.score.instruments.by_id[instrumentId];

          return (
            <PlayerPanelInstrumentListItem
              key={instrumentId}
              playerId={player.id}
              isSelected={isSelected}
              instrument={instrument}
            />
          );
        })}
      </div>
    </div>
  );
};
