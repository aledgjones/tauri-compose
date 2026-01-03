import { FC } from "react";
import { useAppState } from "../../../../../services/state";
import { BottomPanelTab } from "../../../../../services/types";
import { actions } from "../../../../../services/actions";
import { mdiPiano, mdiTuneVertical } from "@mdi/js";
import { IconButton } from "../../../../../ui/icon-button";
import { Sequencer, SequencerToolbar } from "../../../components/sequencer";
import { Mixer, MixerToolbar } from "../../../components/mixer";
import { useBottomPanelState } from "../../../hooks/use-bottom-panel-state";

const SEQUENCE_TABS = [
  { tab: BottomPanelTab.Sequencer, icon: mdiPiano },
  { tab: BottomPanelTab.Mixer, icon: mdiTuneVertical },
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
  const tab = state.ui.sequence.bottom_panel_tab;

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
        <div className="flex flex-1 justify-end">
          <div className="bg-neutral-900 mr-0.5 flex-1" />
          {SEQUENCE_TABS.map((tabConfig) => (
            <IconButton
              key={tabConfig.tab}
              path={tabConfig.icon}
              variant="filled"
              isSelected={tab === tabConfig.tab}
              onClick={() =>
                actions.ui.sequence.bottomPanel.setTab(tabConfig.tab)
              }
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
    </div>
  );
};
