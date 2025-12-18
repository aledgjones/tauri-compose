import { FC } from "react";
import { KEYBOARD_WIDTH } from "../config";
import { DragScroll } from "../../../components/drag-scroll";
import { Keyboard } from "./keyboard";
import { UniqueId } from "../../../services/types";
import { useSlotPattern } from "../hooks/use-slots";
import { useTicks } from "../hooks/use-ticks";
import { useAppState } from "../../../services/state";
import { TickBar } from "./tick-bar";
import { SequencerTrack } from "./sequencer-track";
import { PlaybackLine } from "./playback-line";

type Props = {
  flowId: UniqueId | null;
  instrumentId?: UniqueId | null;
  trackId?: UniqueId | null;
  octaves: number;
};

export const Sequencer: FC<Props> = ({
  flowId,
  instrumentId,
  trackId,
  octaves,
}) => {
  const state = useAppState();
  const flow = flowId ? state.score.flows.by_id[flowId] : null;
  const master = flow?.master_id
    ? state.score.tracks.by_id[flow.master_id]
    : null;
  const track = trackId ? state.score.tracks.by_id[trackId] : null;

  const slots = useSlotPattern(octaves, 1);
  const ticks = useTicks(flow, master);

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
          />
          <PlaybackLine ticks={ticks} height={slots.height} />
        </div>
      </div>
    </DragScroll>
  );
};
