import { FC, useCallback } from "react";
import { Tone, Tool, UniqueId, SelectionType } from "../../../services/types";
import { actions } from "../../../services/actions";
import { Icon } from "../../../ui/icon";
import { mdiChevronLeft, mdiChevronRight } from "@mdi/js";
import { useAppState } from "../../../services/state";
import { lightenColor } from "../../../services/utils";

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

const getShowPitchName = (width: number, name: string) => {
  if (name.length > 2) {
    return width > 40;
  }

  return width > 30;
};

const getShowHandles = (width: number, isSelected: boolean) => {
  if (!isSelected) {
    return false;
  }

  return width > 50;
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
  const appState = useAppState();
  const selection = appState.ui.sequencer.selection;
  const isSelected =
    selection?.selection_type === SelectionType.Tone &&
    selection.id === tone.id;
  const width = tone.duration * tickWidth;
  const name = getPitchName(tone.pitch.int);
  const showPitchName = getShowPitchName(width, name);
  const showHandles = getShowHandles(width, isSelected);
  const backgroundColor = isSelected ? lightenColor(color, 0.2) : color;

  const handleClick = useCallback(async () => {
    switch (tool) {
      case Tool.Select: {
        if (isSelected) {
          await actions.ui.sequencer.clearSelection();
        } else {
          await actions.ui.sequencer.select(SelectionType.Tone, tone.id);
        }
        break;
      }
      case Tool.Erase: {
        await actions.score.tone.remove(trackId, tone.id);
        break;
      }
    }
  }, [tone, tool, trackId, isSelected]);

  return (
    <div
      className="absolute select-none flex items-center cursor-pointer"
      style={{
        backgroundColor,
        bottom: slotHeight * (tone.pitch.int - 12),
        height: slotHeight - 1,
        left: tickWidth * tone.tick + 1,
        width: width - 1,
      }}
      onClick={handleClick}
    >
      {showHandles && (
        <Icon className="-mx-1" path={mdiChevronLeft} color="#fff" />
      )}
      {showPitchName && (
        <div className="text-center flex-1 pl-0.5 text-xs font-medium text-white pointer-events-none">
          {name}
        </div>
      )}
      {showHandles && (
        <Icon className="-mx-1" path={mdiChevronRight} color="#fff" />
      )}
    </div>
  );
};
