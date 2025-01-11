import { mdiAccountCog, mdiPlus } from "@mdi/js";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Panel } from "./panel";
import { PlayerType, SelectionType } from "../../../../../services/types";
import { PlayerPanelItem } from "./player-panel-item";
import { useAppState } from "../../../../../services/state";
import { actions } from "../../../../../services/actions";
import { useEffect, useRef } from "preact/hooks";
import Sortable from "sortablejs";

const onPlayerSettingsPress = () => {
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
};

const onAddInstrumentPress = async () => {
  const playerId = await actions.score.player.create(PlayerType.Solo);
  await actions.ui.setup.setSelection(SelectionType.Player, playerId);
  const definition_id = await actions.pickAnInstrument();
  if (definition_id) {
    const instrumentId = await actions.score.instruments.create(definition_id);
    await actions.score.player.assignInstrument(playerId, instrumentId);
  }
};

export const PlayerPlanel = () => {
  const state = useAppState();

  const ref = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    if (!ref.current) {
      return;
    }

    const sortable = Sortable.create(ref.current, {
      group: "players",
      animation: 150,
      forceFallback: true,
      ghostClass: "opacity-0",
      delay: 150,
      onSort: async (e) => {
        if (e.oldIndex === undefined || e.newIndex === undefined) {
          return;
        }

        await actions.score.player.move(e.oldIndex, e.newIndex);
      },
    });

    return sortable.destroy;
  }, []);

  return (
    <Panel
      className="row-span-2"
      title="Players"
      icons={[
        {
          path: mdiAccountCog,
          onClick: onPlayerSettingsPress,
        },
        {
          path: mdiPlus,
          onClick: onAddInstrumentPress,
        },
      ]}
    >
      <div ref={ref} className="flex flex-col gap-2 px-2 flex-1 overflow-auto">
        {state.score.players.order.map((playerId) => {
          const player = state.score.players.by_id[playerId];

          return (
            <PlayerPanelItem
              key={playerId}
              player={player}
              instruments={state.score.instruments.by_id}
              isExpanded={state.ui.setup.expanded.includes(playerId)}
              isSelected={state.ui.setup.selection?.id === playerId}
            />
          );
        })}
      </div>
    </Panel>
  );
};
