import { mdiAccountCog, mdiPlus } from "@mdi/js";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Panel } from "./panel";
import {
  actions,
  PlayerType,
  useAppState,
} from "../../../../../services/state";
import { PlayerPanelItem } from "./player-panel-item";

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
          onClick: () => {
            actions.score.createPlayer(PlayerType.Solo);
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
