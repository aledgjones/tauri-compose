import { Renderer } from "../../components/renderer";
import { BottomPanel } from "./components/bottom-panel";
import { ZoomWidget } from "../../components/zoom-widget";
import { useAppState } from "../../../../services/state";
import { actions } from "../../../../services/actions";
import MdiIcon from "@mdi/react";
import { mdiChevronDown, mdiChevronUp } from "@mdi/js";
import classNames from "classnames";
import { SelectedToneInfoDisplay } from "../../components/selected-tone-info-display";

export const Write = () => {
  const state = useAppState();
  const minimized = state.ui.write.bottom_panel_minimized;
  const gridTemplateRows = minimized ? "1fr" : "3fr 2fr";

  return (
    <>
      <div
        className="w-screen overflow-hidden px-0.75 grid grid-cols-[36px_1fr_36px] gap-0.5"
        style={{ gridTemplateRows }}
      >
        <div
          className={classNames("bg-neutral-900", { "row-span-2": !minimized })}
        ></div>
        <Renderer />
        <div
          className={classNames("bg-neutral-900", { "row-span-2": !minimized })}
        ></div>
        {!minimized && <BottomPanel />}
      </div>

      <div className="flex justify-center items-center h-7 ml-0.75 mr-0.75 mb-0.75 rounded-b-lg gap-0.5">
        <SelectedToneInfoDisplay />

        <button
          onClick={() => actions.ui.write.bottomPanel.toggle()}
          className="h-full flex-1 flex items-center justify-center w-full text-white  hover:bg-neutral-800 bg-neutral-900"
        >
          <MdiIcon size={1} path={minimized ? mdiChevronUp : mdiChevronDown} />
        </button>

        <ZoomWidget />
      </div>
    </>
  );
};
