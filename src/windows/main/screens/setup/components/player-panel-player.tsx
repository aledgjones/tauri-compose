import { mdiAccount, mdiAccountGroup } from "@mdi/js";
import { Player, PlayerType } from "../../../../../services/state";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { FunctionalComponent } from "preact";

const getIcon = (playerType: PlayerType) => {
  switch (playerType) {
    case PlayerType.Solo:
      return mdiAccount;
    default:
      return mdiAccountGroup;
  }
};

type Props = {
  player: Player;
};

export const PlayerPanelPlayer: FunctionalComponent<Props> = ({ player }) => {
  const isSelected = false;

  return (
    <div
      className={classNames(`mb-2 rounded-lg`, {
        "bg-primary-400 text-white": isSelected,
        "bg-neutral-800 text-white": !isSelected,
      })}
    >
      <div className="flex items-center h-[48px] text-sm pr-1 pl-3">
        <div className="mr-3">
          <Icon path={getIcon(player.player_type)} size={1} />
        </div>

        <p className="truncate flex-1">Player</p>

        {/* {isSelected && (
                  <>
                    <IconButton
                      variant="text"
                      className={fg}
                      onClick={() => {
                        core.remove_player(playerKey);
                        core.ui_setup_clear_selection();
                        core.ui_setup_collapse(playerKey);
                      }}
                    >
                      <Icon path={mdiDelete} size={1} />
                    </IconButton>
                    {isInstrumentAssignable && (
                      <IconButton
                        variant="text"
                        className={fg}
                        onClick={() => onAddInstrument(playerKey)}
                      >
                        <Icon path={mdiPlus} size={1} />
                      </IconButton>
                    )}
                  </>
                )} */}

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

        {/* {!isAssignable && (
                  <IconButton
                    variant="text"
                    className={fg}
                    onClick={() => {
                      if (isExpanded) {
                        core.ui_setup_collapse(playerKey);
                      } else {
                        core.ui_setup_expand(playerKey);
                      }
                    }}
                    onPointerDown={(e) => {
                      e.stopPropagation();
                    }}
                  >
                    <Icon
                      path={isExpanded ? mdiChevronUp : mdiChevronDown}
                      size={1}
                    />
                  </IconButton>
                )} */}
      </div>
    </div>
  );
};
