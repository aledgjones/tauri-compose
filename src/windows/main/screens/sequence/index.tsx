import { BottomPanel } from "./components/bottom-panel";
import { useAppState } from "../../../../services/state";
import { actions } from "../../../../services/actions";
import { mdiChevronDown, mdiChevronUp } from "@mdi/js";
import MdiIcon from "@mdi/react";
import { SelectedToneInfoDisplay } from "../../components/selected-tone-info-display";
import { InstrumentOverview } from "./components/instrument-overview";

export const Sequence = () => {
  const state = useAppState();
  const minimized = state.ui.screens.sequence.bottom_panel_minimized;
  const gridTemplateRows = minimized ? "1fr" : "3fr 2fr";

  return (
    <>
      <div
        className="grid grid-cols-1 gap-0.5 py-0.5 overflow-hidden"
        style={{ gridTemplateRows }}
      >
        {/* Track Overview - shows all instruments with miniature piano rolls */}
        <InstrumentOverview />
        {!minimized && <BottomPanel />}
      </div>

      <div className="flex justify-center items-center h-7 overflow-hidden gap-0.5">
        <SelectedToneInfoDisplay />
        <button
          onClick={() => actions.ui.screens.sequence.bottomPanel.toggle()}
          className="h-full flex-1 flex items-center justify-center w-full text-white hover:bg-neutral-800 rounded-br-xl bg-neutral-900"
        >
          <MdiIcon size={1} path={minimized ? mdiChevronUp : mdiChevronDown} />
        </button>
      </div>
    </>
  );
};
