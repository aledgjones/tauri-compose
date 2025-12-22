import { FC, useCallback } from "react";
import { Tone, Tool, UniqueId } from "../../../services/types";
import { actions } from "../../../services/actions";

type Props = {
  slotHeight: number;
  tickWidth: number;
  tool: Tool;
  trackId: UniqueId;
  tone: Tone;
  color: string;
};

export const SequencerTrackTone: FC<Props> = ({
  slotHeight,
  tickWidth,
  tone,
  tool,
  trackId,
  color,
}) => {
  const handleRemove = useCallback(async () => {
    if (tool === Tool.Erase) {
      await actions.score.tone.remove(trackId, tone.id);
    }
  }, [tone.id, tool, trackId]);

  return (
    <div
      className="absolute"
      style={{
        backgroundColor: color,
        bottom: slotHeight * (tone.pitch.int - 12),
        height: slotHeight - 1,
        left: tickWidth * tone.tick + 1,
        width: tickWidth * tone.duration - 1,
      }}
      onClick={handleRemove}
    />
  );
};
