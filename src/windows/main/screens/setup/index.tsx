import { Renderer } from "../../components/renderer";
import { Panel } from "./components/panel";
import { mdiFileCogOutline, mdiPlus } from "@mdi/js";
import { PlayerPlanel } from "./components/player-panel";
import { FlowPanel } from "./components/flow-panel";

export const Setup = () => {
  return (
    <div class="overflow-hidden p-2 grid grid-rows-[1fr_170px_32px] grid-cols-[360px_1fr_360px] gap-2">
      <PlayerPlanel />
      <Renderer />
      <Panel
        className="row-span-2"
        title="Layouts"
        icons={[
          {
            path: mdiFileCogOutline,
            onClick: () => {},
          },
          {
            path: mdiPlus,
            onClick: () => {},
          },
        ]}
      />
      <FlowPanel />
    </div>
  );
};
