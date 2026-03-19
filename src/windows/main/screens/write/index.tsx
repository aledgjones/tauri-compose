import { Renderer } from "../../components/renderer";
import { BottomPanel } from "./components/bottom-panel";
import { ZoomWidget } from "../../components/zoom-widget";
import { useAppState } from "../../../../services/state";
import { actions } from "../../../../services/actions";
import { LeftPanel } from "./components/left-panel";
import MdiIcon from "@mdi/react";
import { mdiChevronDown, mdiChevronUp } from "@mdi/js";
import classNames from "classnames";
import { SelectedToneInfoDisplay } from "../../components/selected-tone-info-display";
import { useKeyboardShortcuts } from "../../hooks/use-keyboard-shortcuts";

export const Write = () => {
  const state = useAppState();
  const minimized = state.ui.screens.write.bottom_panel_minimized;
  const gridTemplateRows = minimized ? "1fr" : "3fr 2fr";

  useKeyboardShortcuts();

  return (
    <>
      <div
        className="overflow-hidden grid grid-cols-[74px_1fr_36px] gap-0.5 py-0.5"
        style={{ gridTemplateRows }}
      >
        <LeftPanel spanRow={!minimized} />
        <Renderer />
        <div
          className={classNames("bg-neutral-900", { "row-span-2": !minimized })}
        ></div>
        {!minimized && <BottomPanel />}
      </div>

      <div className="flex justify-center items-center h-7 gap-0.5">
        <SelectedToneInfoDisplay />

        <button
          onClick={() => actions.ui.screens.write.bottomPanel.toggle()}
          className="h-full flex-1 flex items-center justify-center w-full text-white  hover:bg-neutral-800 bg-neutral-900"
        >
          <MdiIcon size={1} path={minimized ? mdiChevronUp : mdiChevronDown} />
        </button>

        <ZoomWidget />
      </div>
    </>
  );
};
