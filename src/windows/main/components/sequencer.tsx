import { FC } from "react";
import { KEYBOARD_WIDTH } from "../config";
import { DragScroll } from "../../../components/drag-scroll";
import { Keyboard } from "./keyboard";
import { UniqueId } from "../../../services/types";
import { useSlotPattern } from "../utils/slots";

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
  const slots = useSlotPattern(octaves, 1);
  if (!flowId) {
    return (
      <div className="mx-2 mb-2 flex flex-1 items-center justify-center bg-neutral-800 text-neutral-700">
        <p>You don't have any flows yet...</p>
      </div>
    );
  }

  if (!instrumentId) {
    return (
      <div className="mx-2 mb-2 flex flex-1 items-center justify-center bg-neutral-800 text-neutral-700">
        <p>You don't have any instruments yet...</p>
      </div>
    );
  }

  if (!trackId) {
    return (
      <div className="mx-2 mb-2 flex flex-1 items-center justify-center bg-neutral-800 text-neutral-700">
        <p>This instrument is not included in this flow...</p>
      </div>
    );
  }

  return (
    <DragScroll className="mx-2 mb-2 flex flex-1 bg-neutral-800 relative">
      <div>
        <div className="sticky left-0 z-20" style={{ width: KEYBOARD_WIDTH }}>
          <div className="sticky top-0 z-30 h-[28px] bg-neutral-800 w-full" />
          <Keyboard slots={slots} />
        </div>
        <div
          style={{ paddingLeft: KEYBOARD_WIDTH }}
          className="ignore-y absolute top-0 z-10 min-w-full pr-[80px]"
        >
          {/* <TickBar ticks={ticks} className="sticky top-0 z-30" />
          <SequencerTrack
            tool={tool}
            slots={slots}
            color={color || "#000"}
            flowKey={flowKey}
            instrumentKey={instrumentKey}
            trackKey={trackKey}
            ticks={ticks}
          /> */}
        </div>
      </div>
    </DragScroll>
  );
};
