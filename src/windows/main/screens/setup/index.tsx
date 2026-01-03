import { Renderer } from "../../components/renderer";
import { PlayerPlanel } from "./components/player-panel";
import { FlowPanel } from "./components/flow-panel";
import { LayoutPlanel } from "./components/layout-panel";
import { ZoomWidget } from "../../components/zoom-widget";

export const Setup = () => {
  return (
    <>
      <div className="overflow-hidden px-0.75 grid grid-rows-[1fr_170px] grid-cols-[360px_1fr_360px] gap-0.5">
        <PlayerPlanel />
        <Renderer />
        <LayoutPlanel />
        <FlowPanel />
      </div>

      <div className="flex justify-center items-center h-7 ml-0.75 mr-0.75 mb-0.75 gap-0.5">
        <div className="h-full flex-1 flex items-center justify-center w-full bg-neutral-900 rounded-bl-xl" />
        <ZoomWidget />
      </div>
    </>
  );
};
