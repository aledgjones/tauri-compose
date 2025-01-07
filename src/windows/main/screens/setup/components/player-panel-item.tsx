import {
  mdiAccount,
  mdiAccountGroup,
  mdiChevronDown,
  mdiDelete,
  mdiPlus,
} from "@mdi/js";
import {
  Player,
  PlayerType,
  SelectionType,
} from "../../../../../services/types";
import { Icon } from "../../../../../ui/icon";
import classNames from "classnames";
import { FunctionalComponent } from "preact";
import { IconButton } from "../../../../../ui/icon-button";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { actions } from "../../../../../services/actions";

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
  isExpanded: boolean;
  isSelected: boolean;
};

export const PlayerPanelItem: FunctionalComponent<Props> = ({
  player,
  isSelected,
  isExpanded,
}) => {
  const isAssignable = false;
  const isInstrumentAssignable =
    player.player_type === PlayerType.Solo || player.instruments.length === 0;

  const fg = isSelected ? "text-black" : "text-white";
  const bg = isSelected ? "bg-sky-500" : "bg-neutral-800";

  return (
    <div
      className={classNames(`mb-2 rounded-lg`, fg, bg)}
      onClick={() => {
        actions.ui.setup.setSelection(SelectionType.Player, player.id);
      }}
    >
      <div className="flex items-center h-[48px] text-sm pr-1 pl-3">
        <Icon
          className={classNames("mr-3", fg)}
          path={getIcon(player.player_type)}
          size={1}
        />

        <p className={classNames(`truncate flex-1`, fg)}>Empty-handed Player</p>

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
                onClick={async () => {
                  actions.console.log("[instrument-picker] open");

                  const webview = new WebviewWindow("instrument-picker", {
                    url: "/instrument-picker.html",
                    title: "Instrument Picker",
                    visible: false,
                    titleBarStyle: "overlay",
                    hiddenTitle: true,
                    acceptFirstMouse: true,
                    height: 500,
                    width: 900,
                  });

                  const unlisten = await webview.once(
                    "solocompose://select-instrument",
                    async (e) => {
                      actions.console.log(
                        "[instrument-picker] select instrument"
                      );
                      console.log(player.id, e.payload);
                    }
                  );

                  webview.once("tauri://close-requested", async () => {
                    actions.console.log("[instrument-picker] close");
                    unlisten();
                    webview.close();
                  });
                }}
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
    </div>
  );
};
