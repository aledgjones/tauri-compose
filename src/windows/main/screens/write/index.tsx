import { Renderer } from "../../components/renderer";
import { BottomPanel } from "./components/bottom-panel";
import { useAppState } from "../../../../services/state";

export const Write = () => {
  const state = useAppState();
  const bottomPanelHeight = state.ui.write.bottom_panel_minimized
    ? "52px"
    : "2fr";

  return (
    <div
      className={`overflow-hidden p-2 grid grid-cols-[120px_1fr_120px] gap-2`}
      style={{ gridTemplateRows: `3fr ${bottomPanelHeight} 32px` }}
    >
      <div className="bg-neutral-900 row-span-2"></div>
      <Renderer />
      <div className="bg-neutral-900 row-span-2"></div>
      <BottomPanel />
    </div>
  );
};
