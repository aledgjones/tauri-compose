import { Renderer } from "../../components/renderer";
import { PlayerPlanel } from "./components/player-panel";
import { FlowPanel } from "./components/flow-panel";
import { LayoutPlanel } from "./components/layout-panel";
import { ZoomWidget } from "../../components/zoom-widget";

export const Setup = () => {
  return (
    <>
      <div className="overflow-hidden p-2 grid grid-rows-[1fr_170px] grid-cols-[360px_1fr_360px] gap-2">
        <PlayerPlanel />
        <Renderer />
        <LayoutPlanel />
        <FlowPanel />
      </div>

      <div className="flex justify-center items-center h-7 ml-2 mr-2 mb-2 gap-2">
        <div className="h-full flex-1 flex items-center justify-center w-full bg-neutral-900 rounded-bl-lg" />
        <ZoomWidget />
      </div>
    </>
  );
};
