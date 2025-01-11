import {
  mdiAccount,
  mdiAccountGroup,
  mdiChevronDown,
  mdiDelete,
  mdiPlus,
} from "@mdi/js";
import {
  Instrument,
  Player,
  PlayerType,
  SelectionType,
} from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { IconButton } from "../../../../../ui/icon-button";
import { actions } from "../../../../../services/actions";
import { useEffect, useMemo, useRef } from "preact/hooks";
import { PlayerPanelInstrumentListItem } from "./player-panel-instrument-list-item";
import Sortable from "sortablejs";

const onAddInstrumentPress = async (playerId: number) => {
  const definitionId = await actions.pickAnInstrument();
  if (definitionId) {
    const instrumentId = await actions.score.instrument.create(definitionId);
    await actions.score.player.assignInstrument(playerId, instrumentId);
  }
};

type Props = {
  player: Player;
  instruments: Record<number, Instrument>;
  isExpanded: boolean;
  isSelected: boolean;
};

export const PlayerPanelItem: FunctionalComponent<Props> = ({
  player,
  instruments,
  isSelected,
  isExpanded,
}) => {
  const isAssignable = false;
  const isInstrumentAssignable =
    player.player_type === PlayerType.Solo || player.instruments.length === 0;

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
    const instrumentNames = player.instruments.map((instrumentId) => {
      const instrument = instruments[instrumentId];
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
  }, [player, instruments]);

  const ref = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    if (!ref.current) {
      return;
    }

    const sortable = Sortable.create(ref.current, {
      animation: 150,
      forceFallback: true,
      ghostClass: "opacity-0",
      delay: 150,
      group: "instruments",
      onSort: (e) => {
        if (
          e.oldIndex === undefined ||
          e.newIndex === undefined ||
          !e.from.dataset.playerId ||
          !e.to.dataset.playerId
        ) {
          return;
        }

        // only handle the sort once in the origin player (if across players)
        if (parseInt(e.from.dataset.playerId) !== player.id) {
          return;
        }

        // reorder instruments (possibly across players)
        actions.score.instrument.move(
          e.oldIndex,
          e.newIndex,
          parseInt(e.from.dataset.playerId),
          parseInt(e.to.dataset.playerId)
        );
      },
    });

    return sortable.destroy;
  }, [player.id]);

  return (
    <div
      data-player-id={player.id}
      className={classNames(`rounded-lg`, fg, bg)}
      onPointerDown={() => {
        if (!isSelected) {
          actions.ui.setup.setSelection(SelectionType.Player, player.id);
        }
      }}
    >
      <div className="flex items-center h-[48px] text-sm pr-1 pl-3">
        <Icon className={classNames("mr-3", fg)} path={icon} size={1} />

        <p className={classNames(`truncate flex-1`, fg)}>{playerName}</p>

        {isSelected && (
          <>
            <IconButton
              className={fg}
              variant="standard"
              path={mdiDelete}
              onClick={() => {
                // core.remove_player(playerKey);
                // core.ui_setup_clear_selection();
                // core.ui_setup_collapse(playerKey);
              }}
            />
            {isInstrumentAssignable && (
              <IconButton
                className={fg}
                variant="standard"
                path={mdiPlus}
                onClick={() => onAddInstrumentPress(player.id)}
              />
            )}
          </>
        )}

        {/* {isAssignable && (
          <div className="mr-[-2px]">
            <Checkbox
              color="light-blue"
              checked={isAssigned}
              onPointerDown={(e) => e.stopPropagation()}
              crossOrigin="true"
              onChange={onCheckboxChange}
            />
          </div>
        )} */}

        {!isAssignable && (
          <IconButton
            variant="standard"
            path={mdiChevronDown}
            className={classNames("transition-transform", fg, {
              "rotate-180": isExpanded,
            })}
            onClick={() => {
              if (isExpanded) {
                actions.ui.setup.clearExpanded(player.id);
              } else {
                actions.ui.setup.setExpanded(player.id);
              }
            }}
          />
        )}
      </div>
      <div
        ref={ref}
        data-player-id={player.id}
        className={classNames("flex flex-col p-2 gap-2 min-h-[64px]", {
          hidden: !isExpanded,
        })}
      >
        {player.instruments.map((instrumentId) => {
          const instrument = instruments[instrumentId];
          return (
            <PlayerPanelInstrumentListItem
              isSelected={isSelected}
              instrument={instrument}
            />
          );
        })}
      </div>
    </div>
  );
};
