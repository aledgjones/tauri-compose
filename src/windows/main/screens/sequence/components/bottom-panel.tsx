import { FC, useState } from "react";
import { useAppState } from "../../../../../services/state";
import { BottomPanelTab } from "../../../../../services/types";
import { actions } from "../../../../../services/actions";
import {
  mdiCableData,
  mdiMagnify,
  mdiPiano,
  mdiPlus,
  mdiTuneVertical,
} from "@mdi/js";
import { IconButton } from "../../../../../ui/buttons/icon-button";
import { Input } from "../../../../../ui/input";
import { Sequencer, SequencerToolbar } from "../../../components/sequencer";
import { Mixer, MixerToolbar } from "../../../components/mixer";
import { useBottomPanelState } from "../../../hooks/use-bottom-panel-state";
import { PlaybackRackPanel } from "./playback-rack-panel";

const SEQUENCE_TABS = [
  { tab: BottomPanelTab.Sequencer, icon: mdiPiano, label: "sequencer" },
  { tab: BottomPanelTab.Mixer, icon: mdiTuneVertical, label: "mixer" },
  { tab: BottomPanelTab.Instruments, icon: mdiCableData, label: "instruments" },
];

export const BottomPanel: FC = () => {
  const state = useAppState();
  const [isCreatingSlot, setIsCreatingSlot] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);
  const [slotSearchTerm, setSlotSearchTerm] = useState("");
  const {
    instruments,
    instrumentOptions,
    flowOptions,
    flowId,
    instrumentId,
    tracks,
    trackId,
    tool,
    snapValue,
  } = useBottomPanelState();
  const tab = state.ui.screens.sequence.bottom_panel_tab;

  const handleCreateSlot = async () => {
    if (isCreatingSlot) {
      return;
    }

    setIsCreatingSlot(true);
    setCreateError(null);

    try {
      await actions.score.playback.slot.create();
    } catch (error) {
      setCreateError(
        error instanceof Error ? error.message : "Failed to create slot.",
      );
    } finally {
      setIsCreatingSlot(false);
    }
  };

  return (
    <div className="flex flex-col min-h-0 overflow-hidden">
      <div className="flex pb-0.5 h-10 items-center z-50">
        {tab === BottomPanelTab.Sequencer && (
          <SequencerToolbar
            tool={tool}
            flowId={flowId}
            flowOptions={flowOptions}
            showFlowPicker={false}
            instrumentId={instrumentId}
            instruments={instrumentOptions}
            trackId={trackId}
            tracks={tracks}
            snapValue={snapValue}
          />
        )}
        {tab === BottomPanelTab.Mixer && (
          <MixerToolbar
            instruments={instruments}
            resolvedPlaybackRouteByInstrumentId={
              state.resolved_playback_route_by_instrument_id
            }
          />
        )}
        {tab === BottomPanelTab.Instruments && (
          <div className="flex flex-1 gap-0.5">
            <IconButton
              path={mdiPlus}
              variant="interactive"
              onClick={handleCreateSlot}
              isDisabled={isCreatingSlot}
              title="Add slot"
            />
            <div className="bg-neutral-900 flex flex-1 items-center px-2">
              <Input
                type="text"
                variant="dark"
                leadingIcon={mdiMagnify}
                value={slotSearchTerm}
                onChange={setSlotSearchTerm}
                placeholder="Search plugins..."
                className="flex-1"
              />
            </div>
          </div>
        )}
        <div className="flex flex-1 justify-end gap-0.5">
          <div className="bg-neutral-900 flex-1" />
          {SEQUENCE_TABS.map((tabConfig) => (
            <IconButton
              key={tabConfig.tab}
              path={tabConfig.icon}
              variant="interactive"
              isSelected={tab === tabConfig.tab}
              aria-label={`Show ${tabConfig.label} panel`}
              title={`Show ${tabConfig.label} panel`}
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
      {tab === BottomPanelTab.Mixer && (
        <Mixer
          instruments={instruments}
          resolvedPlaybackRouteByInstrumentId={
            state.resolved_playback_route_by_instrument_id
          }
        />
      )}
      {tab === BottomPanelTab.Instruments && (
        <PlaybackRackPanel
          createError={createError}
          searchTerm={slotSearchTerm}
        />
      )}
    </div>
  );
};
