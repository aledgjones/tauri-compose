import { FC, useMemo } from "react";
import { useAppState, useSamplerState } from "../../../services/state";
import { Select, SelectOption } from "../../../ui/select";
import { BottomPanelTab, Tool, UniqueId } from "../../../services/types";
import { actions } from "../../../services/actions";
import {
  mdiBoxCutter,
  mdiChevronDown,
  mdiChevronUp,
  mdiCursorDefault,
  mdiEraser,
  mdiFormatListBulletedSquare,
  mdiPen,
  mdiPiano,
  mdiTuneVertical,
} from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import { Sequencer } from "./sequencer";
import { SnapPicker } from "../../../components/snap-picker";
import { Mixer } from "./mixer";

const ALPHABET = ["A", "B"];
const TRACK_LABEL = ["Up-stem", "Down-stem"];

interface SequencerToolbarProps {
  tool: Tool;
  instrumentId: UniqueId | null;
  instruments: SelectOption<UniqueId>[];
  trackId: UniqueId | null;
  tracks: SelectOption<UniqueId>[];
  snapValue: any;
}

const SequencerToolbar: FC<SequencerToolbarProps> = ({
  tool,
  instrumentId,
  instruments,
  trackId,
  tracks,
  snapValue,
}) => (
  <>
    <div className="flex flex-1">
      <IconButton
        path={mdiCursorDefault}
        variant="filled"
        isSelected={tool === Tool.Select}
        onClick={() => actions.ui.sequencer.setTool(Tool.Select)}
      />
      <IconButton
        path={mdiPen}
        variant="filled"
        isSelected={tool === Tool.Write}
        onClick={() => actions.ui.sequencer.setTool(Tool.Write)}
      />
      <IconButton
        path={mdiBoxCutter}
        variant="filled"
        isSelected={tool === Tool.Slice}
        onClick={() => actions.ui.sequencer.setTool(Tool.Slice)}
      />
      <IconButton
        className="mr-2"
        path={mdiEraser}
        variant="filled"
        isSelected={tool === Tool.Erase}
        onClick={() => actions.ui.sequencer.setTool(Tool.Erase)}
      />
      <SnapPicker value={snapValue} onChange={actions.ui.sequencer.setSnap} />
    </div>
    <div className="flex flex-1 justify-center">
      {instrumentId && instruments.length > 0 && (
        <Select
          className="mr-1"
          variant="filled"
          value={instrumentId}
          options={instruments}
          onChange={actions.ui.sequencer.setInstrument}
        />
      )}
      {trackId && tracks.length > 0 && (
        <Select
          variant="filled"
          value={trackId}
          options={tracks}
          onChange={actions.ui.sequencer.setTrack}
        />
      )}
    </div>
  </>
);

interface MixerToolbarProps {}

const MixerToolbar: FC<MixerToolbarProps> = () => (
  <>
    <div className="flex flex-1">
      {/* Mixer-specific controls can go here */}
    </div>
  </>
);

interface TabButtonsProps {
  tab: BottomPanelTab;
}

const TabButtons: FC<TabButtonsProps> = ({ tab }) => (
  <div className="flex flex-1 justify-end">
    <IconButton
      path={mdiFormatListBulletedSquare}
      variant="filled"
      isSelected={tab === BottomPanelTab.Properties}
      onClick={() =>
        actions.ui.write.bottomPanel.setTab(BottomPanelTab.Properties)
      }
    />
    <IconButton
      path={mdiPiano}
      variant="filled"
      isSelected={tab === BottomPanelTab.Sequencer}
      onClick={() =>
        actions.ui.write.bottomPanel.setTab(BottomPanelTab.Sequencer)
      }
    />
    <IconButton
      className="mr-2"
      path={mdiTuneVertical}
      variant="filled"
      isSelected={tab === BottomPanelTab.Mixer}
      onClick={() => actions.ui.write.bottomPanel.setTab(BottomPanelTab.Mixer)}
    />
    <IconButton
      path={mdiChevronDown}
      variant="filled"
      onClick={() => actions.ui.write.bottomPanel.toggle()}
    />
  </div>
);

export const BottomPanel: FC = () => {
  const state = useAppState();
  const samplerState = useSamplerState();

  const instruments = useMemo(() => {
    return state.score.players.order.flatMap((playerId) => {
      const player = state.score.players.by_id[playerId];
      return player.instrument_ids.map((instrumentId) => {
        return state.score.instruments.by_id[instrumentId];
      });
    });
  }, [state]);

  const instrumentOptions: SelectOption<UniqueId>[] = useMemo(() => {
    return instruments.map((instrument) => ({
      value: instrument.id,
      label:
        instrument.long_name + (instrument.count ? ` ${instrument.count}` : ""),
    }));
  }, [instruments]);

  const flowId = useMemo(() => {
    return state.score.flows.order[0] || null;
  }, [state]);

  const instrumentId = useMemo(() => {
    if (instruments.length === 0) {
      return null;
    }

    const id = state.ui.sequencer.instrument;

    // check the selection still exists
    if (id && instruments.some((instrument) => instrument.id === id)) {
      return id;
    }

    // fallback to the first instrument
    return instruments[0].id;
  }, [instruments, state.ui.sequencer.instrument]);

  const tracks = useMemo(() => {
    if (!flowId || !instrumentId) {
      return [];
    }

    const flow = state.score.flows.by_id[flowId];
    const instrument = state.score.instruments.by_id[instrumentId];

    if (!flow || !instrument) {
      return [];
    }

    const stave_ids = instrument.stave_ids;

    return stave_ids.flatMap((staveId, i) => {
      const stave = flow.staves.by_id[staveId];

      if (!stave) {
        return [];
      }

      return stave.track_ids.map((trackId, ii) => {
        const staffLabel =
          stave_ids.length > 1 ? `Staff (${ALPHABET[i]}), ` : ``;
        const trackLabel = `${TRACK_LABEL[ii % 2]} voice ${
          Math.floor(ii / 2) + 1
        }`;

        return {
          value: trackId,
          label: [staffLabel, trackLabel].join(""),
        };
      });
    });
  }, [
    flowId,
    instrumentId,
    state.score.flows.by_id,
    state.score.instruments.by_id,
  ]);

  const trackId = useMemo(() => {
    if (tracks.length === 0) {
      return null;
    }

    const id = state.ui.sequencer.track;

    // check the selection still exists
    if (
      id &&
      tracks.some((option: SelectOption<UniqueId>) => option.value === id)
    ) {
      return id;
    }

    // fallback to the first instrument
    return tracks[0].value;
  }, [state.ui.sequencer.track, tracks]);

  const tool = state.ui.sequencer.tool;
  const tab = state.ui.write.bottom_panel_tab;
  const minimised = state.ui.write.bottom_panel_minimized;

  if (minimised) {
    return (
      <div className="bg-neutral-900 flex items-center justify-end p-2">
        <IconButton
          path={mdiChevronUp}
          variant="filled"
          onClick={() => actions.ui.write.bottomPanel.toggle()}
        />
      </div>
    );
  }

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
            snapValue={state.ui.sequencer.snap}
          />
        )}
        {tab === BottomPanelTab.Mixer && <MixerToolbar />}
        <TabButtons tab={tab} />
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
