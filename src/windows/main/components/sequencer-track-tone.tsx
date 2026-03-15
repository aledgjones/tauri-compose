import { FC } from "react";
import { Tone } from "../../../services/types";
import { Icon } from "../../../ui/icon";
import { mdiMenuLeft, mdiMenuRight } from "@mdi/js";
import { useAppState } from "../../../services/state";
import { lightenColor } from "../../../services/utils";
import { getPitchName } from "../../../services/pitch-utils";
import { TokenizedText } from "../../../ui/text";
import classNames from "classnames";

const getShowPitchName = (width: number, name: string, isSelected: boolean) => {
  if (name.length > 2) {
    return isSelected ? width > 40 : width > 30;
  }

  return isSelected ? width > 30 : width > 30;
};

const getShowHandles = (width: number, isSelected: boolean) => {
  if (!isSelected) {
    return false;
  }

  return width > 20;
};

type Props = {
  slotHeight: number;
  tickWidth: number;
  tone: Tone;
  color: string;
  originTick?: number;
  tickScale?: number;
  tupletId?: number;
};

export const SequencerTrackTone: FC<Props> = ({
  slotHeight,
  tickWidth,
  tone,
  color,
  originTick = 0,
  tickScale = 1,
  tupletId,
}) => {
  const appState = useAppState();
  const selection = appState.ui.sequencer.selection;
  const isSelected =
    selection !== null &&
    selection.type === "Tone" &&
    selection.tone_id === tone.id &&
    (selection.tuplet_id ?? null) === (tupletId ?? null);
  const scaledTickWidth = tickWidth * tickScale;
  const width = tone.duration * scaledTickWidth;
  const name = getPitchName(tone.pitch);
  const showPitchName = getShowPitchName(width, name, isSelected);
  const showHandles = getShowHandles(width, isSelected);
  const backgroundColor = isSelected ? lightenColor(color, 0.5) : color;

  return (
    <div
      className="absolute select-none flex justify-between items-center cursor-pointer"
      style={{
        backgroundColor,
        bottom: slotHeight * (tone.pitch.int - 12),
        height: slotHeight - 1,
        left: tickWidth * originTick + scaledTickWidth * tone.tick + 1,
        width: width - 1,
      }}
      data-tone-id={tone.id}
      {...(tupletId !== undefined ? { "data-tuplet-id": tupletId } : {})}
    >
      {showHandles && (
        <Icon className="-mx-1.5" path={mdiMenuLeft} color="black" />
      )}
      {showPitchName && (
        <div
          className={classNames(
            "flex-1 text-[11px] font-medium pointer-events-none text-black",
            { "px-1": !showHandles },
          )}
        >
          <TokenizedText>{name}</TokenizedText>
        </div>
      )}
      {showHandles && (
        <Icon className="-mx-1.5" path={mdiMenuRight} color="black" />
      )}
    </div>
  );
};
