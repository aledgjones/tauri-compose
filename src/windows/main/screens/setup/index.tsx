import { Renderer } from "../../components/renderer";
import { PlayerPlanel } from "./components/player-panel";
import { FlowPanel } from "./components/flow-panel";
import { LayoutPlanel } from "./components/layout-panel";

export const Setup = () => {
  return (
    <div className="overflow-hidden p-2 grid grid-rows-[1fr_170px_32px] grid-cols-[360px_1fr_360px] gap-2">
      <PlayerPlanel />
      <Renderer />
      <LayoutPlanel />
      <FlowPanel />
    </div>
  );
};
