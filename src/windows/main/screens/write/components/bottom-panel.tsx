import { FC } from "react";
import { useAppState, useSamplerState } from "../../../../../services/state";
import { BottomPanelTab } from "../../../../../services/types";
import { actions } from "../../../../../services/actions";
import {
  mdiFormatListBulletedSquare,
  mdiPiano,
  mdiTuneVertical,
} from "@mdi/js";
import { IconButton } from "../../../../../ui/icon-button";
import { Sequencer, SequencerToolbar } from "../../../components/sequencer";
import { Mixer, MixerToolbar } from "../../../components/mixer";
import { useBottomPanelState } from "../../../hooks/use-bottom-panel-state";

const WRITE_TABS = [
  { tab: BottomPanelTab.Properties, icon: mdiFormatListBulletedSquare },
  { tab: BottomPanelTab.Sequencer, icon: mdiPiano },
  { tab: BottomPanelTab.Mixer, icon: mdiTuneVertical },
];

export const BottomPanel: FC = () => {
  const state = useAppState();
  const samplerState = useSamplerState();
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
  const tab = state.ui.write.bottom_panel_tab;

  return (
    <div className="bg-neutral-900 flex flex-col min-h-0 overflow-hidden">
      <div className="flex p-2 h-13 items-center z-50">
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
        {tab === BottomPanelTab.Mixer && <MixerToolbar />}
        <div className="flex flex-1 justify-end">
          {WRITE_TABS.map((tabConfig) => (
            <IconButton
              key={tabConfig.tab}
              path={tabConfig.icon}
              variant="filled"
              isSelected={tab === tabConfig.tab}
              onClick={() => actions.ui.write.bottomPanel.setTab(tabConfig.tab)}
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
      {tab === BottomPanelTab.Mixer && (
        <Mixer instruments={instruments} samplerState={samplerState} />
      )}
    </div>
  );
};
