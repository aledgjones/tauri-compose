import { Renderer } from "../../components/renderer";
import { BottomPanel } from "./components/bottom-panel";

export const Write = () => {
  return (
    <div className="overflow-hidden p-2 grid grid-rows-[3fr_2fr_32px] grid-cols-[120px_1fr_120px] gap-2">
      <div className="bg-neutral-900 row-span-2"></div>
      <Renderer />
      <div className="bg-neutral-900 row-span-2"></div>
      <BottomPanel />
    </div>
  );
};
