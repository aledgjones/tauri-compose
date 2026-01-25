import { FC, useMemo } from "react";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import { Fader } from "./fader";
import { Instrument } from "../../../services/types";
import { desaturateColor } from "../../../services/utils";
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
  // Desaturate the color for background and fader border
  const desaturatedColor = useMemo(() => {
    return color ? desaturateColor(color, 0.25, 0.4) : undefined;
  }, [color]);

  return (
    <div
      className="flex flex-col w-24 h-full"
      style={{ backgroundColor: desaturatedColor }}
    >
      {/* Fader */}
      <div className="flex flex-1 w-full items-center justify-center p-8 pb-5">
        <Fader
          volume={volume}
          disabled={disabled}
          amplitude={0}
          color={color}
          borderColor={desaturatedColor}
          onChange={(val) => onVolumeChange(Math.round((val / 100) * 127))}
        />
      </div>

      {/* Solo and Mute buttons */}
      <div className="flex items-center gap-2 p-2">
        <IconButton
          path={mdiSizeS}
          variant="filled"
          size={0.9}
          isSelected={isSoloed}
          onClick={onToggleSolo}
        />
        <IconButton
          path={mdiSizeM}
          variant="filled"
          size={0.9}
          isSelected={isMuted}
          onClick={onToggleMute}
        />
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
