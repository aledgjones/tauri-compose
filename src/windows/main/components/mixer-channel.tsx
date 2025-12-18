import { FC } from "react";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import { Fader } from "./fader";
import { Instrument } from "../../../services/types";

type MixerProps = {
  instrument: Instrument;
  disabled?: boolean;
  amplitude?: number;
  color?: string;
  onVolumeChange: (volume: number) => void;
  onToggleMute: () => void;
  onToggleSolo: () => void;
};

export const MixerChannel: FC<MixerProps> = ({
  instrument,
  disabled = false,
  amplitude = 0,
  color,
  onVolumeChange,
  onToggleMute,
  onToggleSolo,
}) => {
  return (
    <div className="flex flex-col w-24 h-full bg-neutral-800">
      {/* Fader */}
      <div className="flex flex-1 w-full items-center justify-center p-8 pb-5">
        <Fader
          volume={Math.round((instrument.volume / 127) * 100)}
          disabled={disabled}
          amplitude={amplitude}
          color={color}
          onChange={(val) => onVolumeChange(Math.round((val / 100) * 127))}
        />
      </div>

      {/* Solo and Mute buttons */}
      <div className="flex items-center gap-2 p-2">
        <IconButton
          path={mdiSizeS}
          variant="outline"
          size={0.9}
          isSelected={instrument.solo}
          onClick={onToggleSolo}
        />
        <IconButton
          path={mdiSizeM}
          variant="outline"
          size={0.9}
          isSelected={instrument.mute}
          onClick={onToggleMute}
        />
      </div>

      {/* Instrument name and color bar */}
      <div className="w-full" style={{ backgroundColor: color }}>
        <div className="text-center text-xs text-black px-2 py-1 truncate">
          {instrument.short_name}
        </div>
      </div>
    </div>
  );
};
