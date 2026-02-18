import { FC } from "react";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { IconButton } from "../../../ui/buttons/icon-button";
import { Fader } from "./fader";
import { Instrument } from "../../../services/types";
import { TokenizedText } from "../../../ui/text";

type MixerProps = {
  instrument: Instrument;
  disabled?: boolean;
  color?: string;
  volume: number;
  isMuted: boolean;
  isSoloed: boolean;
  onVolumeChange: (volume: number) => void;
  onToggleMute: () => void;
  onToggleSolo: () => void;
};

export const MixerChannel: FC<MixerProps> = ({
  instrument,
  disabled = false,
  color,
  volume,
  isMuted,
  isSoloed,
  onVolumeChange,
  onToggleMute,
  onToggleSolo,
}) => {
  return (
    <div className="flex flex-col w-24 h-full bg-neutral-900">
      <div className="p-2 pb-0">
        <div className="p-2 text-sky-500 text-xs text-center bg-neutral-950">
          C
        </div>
      </div>
      <div className="flex flex-1">
        {/* Solo and Mute buttons */}
        <div className="flex flex-col gap-2 pl-2 py-2">
          <IconButton
            path={mdiSizeS}
            variant="filled-dark"
            size={0.9}
            isSelected={isSoloed}
            onClick={onToggleSolo}
          />
          <IconButton
            path={mdiSizeM}
            variant="filled-dark"
            size={0.9}
            isSelected={isMuted}
            onClick={onToggleMute}
          />
        </div>

        {/* Fader */}
        <div className="flex flex-1 items-center justify-center px-2 py-4">
          <Fader
            volume={volume}
            disabled={disabled}
            amplitude={0}
            color={color}
            onChange={(val) => onVolumeChange(Math.round((val / 100) * 127))}
          />
        </div>
      </div>

      {/* Instrument name and color bar */}
      <div className="w-full" style={{ backgroundColor: color }}>
        <div className="text-center text-xs text-black px-2 py-1 truncate">
          <TokenizedText>{instrument.short_name}</TokenizedText>
        </div>
      </div>
    </div>
  );
};
