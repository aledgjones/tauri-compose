import { FC, useState } from "react";
import { useAppState } from "../../../../../services/state";
import { BottomPanelTab } from "../../../../../services/types";
import { actions } from "../../../../../services/actions";
import { mdiCableData, mdiPiano, mdiTuneVertical } from "@mdi/js";
import { IconButton } from "../../../../../ui/icon-button";
import { Sequencer, SequencerToolbar } from "../../../components/sequencer";
import { Mixer, MixerToolbar } from "../../../components/mixer";
import {
  Instruments,
  InstrumentsToolbar,
} from "../../../components/instruments";
import { useBottomPanelState } from "../../../hooks/use-bottom-panel-state";

const SEQUENCE_TABS = [
  { tab: BottomPanelTab.Sequencer, icon: mdiPiano },
  { tab: BottomPanelTab.Mixer, icon: mdiTuneVertical },
  { tab: BottomPanelTab.Instruments, icon: mdiCableData },
];

export const BottomPanel: FC = () => {
  const state = useAppState();
  const {
    instruments,
    instrumentOptions,
    flowId,
    instrumentId,
    tracks,
    trackId,
    tool,
    snapValue,
  } = useBottomPanelState();
  const tab = state.ui.screens.sequence.bottom_panel_tab;
  const [searchTerm, setSearchTerm] = useState("");

  return (
    <div className="flex flex-col min-h-0 overflow-hidden">
      <div className="flex pb-0.5 h-10 items-center z-50">
        {tab === BottomPanelTab.Sequencer && (
          <SequencerToolbar
            tool={tool}
            instrumentId={instrumentId}
            instruments={instrumentOptions}
            trackId={trackId}
            tracks={tracks}
            snapValue={snapValue}
          />
        )}
        {tab === BottomPanelTab.Mixer && (
          <MixerToolbar instruments={instruments} />
        )}
        {tab === BottomPanelTab.Instruments && (
          <InstrumentsToolbar
            searchTerm={searchTerm}
            onSearchTermChange={setSearchTerm}
          />
        )}
        <div className="flex flex-1 justify-end gap-0.5">
          <div className="bg-neutral-900 flex-1" />
          {SEQUENCE_TABS.map((tabConfig) => (
            <IconButton
              key={tabConfig.tab}
              path={tabConfig.icon}
              variant="interactive"
              isSelected={tab === tabConfig.tab}
              onClick={() => {
                actions.ui.screens.sequence.bottomPanel.setTab(tabConfig.tab);
              }}
            />
          ))}
        </div>
      </div>
      {tab === BottomPanelTab.Sequencer && (
        <Sequencer
          octaves={10}
          flowId={flowId}
          instrumentId={instrumentId}
          trackId={trackId}
          instruments={instruments}
        />
      )}
      {tab === BottomPanelTab.Mixer && <Mixer instruments={instruments} />}
      {tab === BottomPanelTab.Instruments && (
        <Instruments searchTerm={searchTerm} />
      )}
    </div>
  );
};
