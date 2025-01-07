import { mdiAccountCog, mdiPlus } from "@mdi/js";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Panel } from "./panel";
import { PlayerType, SelectionType } from "../../../../../services/types";
import { PlayerPanelItem } from "./player-panel-item";
import { useAppState } from "../../../../../services/state";
import { actions } from "../../../../../services/actions";

export const PlayerPlanel = () => {
  const state = useAppState();

  return (
    <Panel
      className="row-span-2"
      title="Players"
      icons={[
        {
          path: mdiAccountCog,
          onClick: () => {
            const webview = new WebviewWindow("player-settings", {
              url: "/player-settings.html",
              title: "Player Settings",
              visible: false,
              titleBarStyle: "overlay",
              hiddenTitle: true,
              acceptFirstMouse: true,
              height: 500,
              width: 750,
            });

            webview.setFocus();
          },
        },
        {
          path: mdiPlus,
          onClick: async () => {
            const playerId = await actions.score.createPlayer(PlayerType.Solo);
            await actions.ui.setup.setSelection(SelectionType.Player, playerId);

            actions.console.log("[instrument-picker] open");

            const webview = new WebviewWindow("instrument-picker", {
              url: "/instrument-picker.html",
              title: "Instrument Picker",
              visible: false,
              titleBarStyle: "overlay",
              hiddenTitle: true,
              acceptFirstMouse: true,
              height: 500,
              width: 750,
            });

            const unlisten = await webview.once(
              "solocompose://select-instrument",
              async (e) => {
                actions.console.log("[instrument-picker] select instrument");
                console.log(playerId, e.payload);
              }
            );

            webview.once("tauri://close-requested", async () => {
              actions.console.log("[instrument-picker] close");
              unlisten();
              webview.close();
            });
          },
        },
      ]}
    >
      <div className="px-2 flex-1 overflow-auto">
        {state.score.players.order.map((playerId) => {
          const player = state.score.players.by_id[playerId];

          return (
            <PlayerPanelItem
              key={playerId}
              player={player}
              isExpanded={state.ui.setup.expanded.includes(playerId)}
              isSelected={state.ui.setup.selection?.id === playerId}
            />
          );
        })}
      </div>
    </Panel>
  );
};
