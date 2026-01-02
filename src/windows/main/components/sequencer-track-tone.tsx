import { FC, MouseEvent, useCallback } from "react";
import { Tone, Tool, UniqueId } from "../../../services/types";
import { actions } from "../../../services/actions";

const getPitchName = (pitchInt: number): string => {
  const noteNames = [
    "C",
    "C#",
    "D",
    "D#",
    "E",
    "F",
    "F#",
    "G",
    "G#",
    "A",
    "A#",
    "B",
  ];
  const octave = Math.floor(pitchInt / 12) - 1;
  const noteIndex = pitchInt % 12;
  return `${noteNames[noteIndex]}${octave}`;
};

const showPitchName = (width: number, name: string) => {
  if (name.length > 2) {
    return width > 32;
  } else {
    return width > 26;
  }
};

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
  const width = tone.duration * tickWidth;
  const name = getPitchName(tone.pitch.int);
  const showName = showPitchName(width, name);

  const handleClick = useCallback(
    async (e: MouseEvent<HTMLDivElement>) => {
      switch (tool) {
        case Tool.Erase: {
          await actions.score.tone.remove(trackId, tone.id);
          break;
        }
      }
    },
    [tone, tool, trackId],
  );

  return (
    <div
      className="absolute select-none"
      style={{
        backgroundColor: color,
        bottom: slotHeight * (tone.pitch.int - 12),
        height: slotHeight - 1,
        left: tickWidth * tone.tick + 1,
        width: width - 1,
      }}
      onClick={handleClick}
    >
      {showName && (
        <div className="absolute inset-y-0 left-0 flex items-center pl-1 text-xs font-medium text-white pointer-events-none">
          {name}
        </div>
      )}
    </div>
  );
};
