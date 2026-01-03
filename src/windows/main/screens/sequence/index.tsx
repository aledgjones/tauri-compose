import { BottomPanel } from "./components/bottom-panel";
import { useAppState } from "../../../../services/state";
import { actions } from "../../../../services/actions";
import { mdiChevronDown, mdiChevronUp } from "@mdi/js";
import MdiIcon from "@mdi/react";
import { NotFound } from "../../../../components/not-found";
import { SelectedToneInfoDisplay } from "../../components/selected-tone-info-display";

export const Sequence = () => {
  const state = useAppState();
  const minimized = state.ui.sequence.bottom_panel_minimized;
  const gridTemplateRows = minimized ? "1fr" : "3fr 2fr";

  return (
    <>
      <div
        className="w-screen p-2 grid grid-cols-1 gap-2 overflow-hidden"
        style={{ gridTemplateRows }}
      >
        {/* Track Overview Placeholder - will show track overviews eventually */}
        <div className="bg-neutral-900 flex items-center justify-center text-neutral-400">
          <NotFound />
        </div>
        {!minimized && <BottomPanel />}
      </div>

      <div className="flex justify-center items-center h-7 ml-2 mr-2 mb-2 rounded-b-lg overflow-hidden gap-2">
        <SelectedToneInfoDisplay />
        <button
          onClick={() => actions.ui.sequence.bottomPanel.toggle()}
          className="h-full flex-1 flex items-center justify-center w-full text-white  hover:bg-neutral-800 rounded-br-lg bg-neutral-900"
        >
          <MdiIcon size={1} path={minimized ? mdiChevronUp : mdiChevronDown} />
        </button>
      </div>
    </>
  );
};
