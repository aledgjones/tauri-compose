import { FC, useMemo } from "react";
import { KEYBOARD_WIDTH } from "../config";
import { DragScroll } from "../../../components/drag-scroll";
import { Keyboard } from "./keyboard";
import { Tool, UniqueId } from "../../../services/types";
import { useSlotPattern } from "../hooks/use-slots";
import { useTicks } from "../hooks/use-ticks";
import { useAppState } from "../../../services/state";
import { TickBar } from "./tick-bar";
import { SequencerTrack } from "./sequencer-track";
import { PlaybackLine } from "./playback-line";
import { generateRainbowColors } from "../../../services/utils";
import { Instrument } from "../../../services/types";
import { IconButton } from "../../../ui/icon-button";
import { mdiBoxCutter, mdiCursorDefault, mdiEraser, mdiPen } from "@mdi/js";
import { actions } from "../../../services/actions";
import { SnapPicker } from "../../../components/snap-picker";
import { Select, SelectOption } from "../../../ui/select";

interface SequencerToolbarProps {
  tool: Tool;
  instrumentId: UniqueId | null;
  instruments: SelectOption<UniqueId>[];
  trackId: UniqueId | null;
  tracks: SelectOption<UniqueId>[];
  snapValue: any;
}

export const SequencerToolbar: FC<SequencerToolbarProps> = ({
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
        className="mr-0.5"
        path={mdiEraser}
        variant="filled"
        isSelected={tool === Tool.Erase}
        onClick={() => actions.ui.sequencer.setTool(Tool.Erase)}
      />
      <SnapPicker value={snapValue} onChange={actions.ui.sequencer.setSnap} />
      <div className="bg-neutral-900 ml-0.5 flex-1" />
    </div>
    <div className="flex flex-1 justify-center">
      <div className="bg-neutral-900 mr-0.5 flex-1" />
      {instrumentId && instruments.length > 0 && (
        <Select
          className="mr-0.5"
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
      <div className="bg-neutral-900 ml-0.5 flex-1" />
    </div>
  </>
);

type Props = {
  flowId: UniqueId | null;
  instrumentId?: UniqueId | null;
  trackId?: UniqueId | null;
  octaves: number;
  instruments: Instrument[];
};

export const Sequencer: FC<Props> = ({
  flowId,
  instrumentId,
  trackId,
  octaves,
  instruments,
}) => {
  const state = useAppState();
  const flow = flowId ? state.score.flows.by_id[flowId] : null;
  const master = flow?.master_id
    ? state.score.tracks.by_id[flow.master_id]
    : null;
  const track = trackId ? state.score.tracks.by_id[trackId] : null;

  const slots = useSlotPattern(octaves, 1);
  const ticks = useTicks(flow, master);

  const instrumentColors = useMemo(() => {
    return generateRainbowColors(instruments.length);
  }, [instruments.length]);

  const color = useMemo(() => {
    const index = instruments.findIndex((inst) => inst.id === instrumentId);

    return instrumentColors[index];
  }, [instruments, instrumentId, instrumentColors]);

  if (!flowId || !flow) {
    return (
      <div className="flex items-center justify-center w-full h-full bg-neutral-900 relative text-neutral-700">
        <p>You don't have any flows yet...</p>
      </div>
    );
  }

  if (!instrumentId) {
    return (
      <div className="flex items-center justify-center w-full h-full bg-neutral-900 relative text-neutral-700">
        <p>You don't have any instruments yet...</p>
      </div>
    );
  }

  if (!trackId || !track) {
    return (
      <div className="flex items-center justify-center w-full h-full bg-neutral-900 relative text-neutral-700">
        <p>This instrument is not included in this flow...</p>
      </div>
    );
  }

  return (
    <DragScroll className="w-full h-full bg-neutral-900 relative">
      <div
        className="ignore-x sticky left-0 z-20"
        style={{ width: KEYBOARD_WIDTH }}
      >
        <div className="sticky top-0 z-30 h-7 bg-neutral-800 w-full" />
        <Keyboard slots={slots} />
      </div>
      <div
        style={{ paddingLeft: KEYBOARD_WIDTH }}
        className="ignore-y absolute top-0 z-10 min-w-full"
      >
        <TickBar ticks={ticks} className="sticky top-0 z-30 bg-neutral-800" />
        <div className="relative">
          <SequencerTrack
            flow={flow}
            track={track}
            snapDuration={state.ui.sequencer.snap}
            ticks={ticks}
            slots={slots}
            tool={state.ui.sequencer.tool}
            instrumentId={instrumentId}
            color={color}
          />
          <PlaybackLine ticks={ticks} height={slots.height} />
        </div>
      </div>
    </DragScroll>
  );
};
