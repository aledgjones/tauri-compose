import { Renderer } from "../../components/renderer";
import { BottomPanel } from "./components/bottom-panel";
import { useAppState } from "../../../../services/state";
import { actions } from "../../../../services/actions";
import MdiIcon from "@mdi/react";
import { mdiChevronDown, mdiChevronUp } from "@mdi/js";
import classNames from "classnames";

export const Write = () => {
  const state = useAppState();
  const minimized = state.ui.write.bottom_panel_minimized;
  const gridTemplateRows = minimized ? "1fr" : "3fr 2fr";

  return (
    <>
      <div
        className="w-screen overflow-hidden p-2 grid grid-cols-[120px_1fr_120px] gap-2"
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
      <div className="flex justify-center items-center h-7 ml-2 mr-2 mb-2 rounded-b-lg bg-neutral-900">
        <button
          onClick={() => actions.ui.write.bottomPanel.toggle()}
          className=" h-full flex-1 flex items-center justify-center w-full text-white  hover:bg-neutral-800 rounded-b-lg"
        >
          <MdiIcon size={1} path={minimized ? mdiChevronUp : mdiChevronDown} />
        </button>
      </div>
    </>
  );
};
