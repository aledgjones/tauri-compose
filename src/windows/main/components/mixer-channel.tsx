import { FC, useEffect, useState, useRef, useMemo } from "react";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import { Fader } from "./fader";
import { sampler, transport } from "../../../sampler";
import { Instrument } from "../../../services/types";
import { desaturateColor } from "../../../services/utils";

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
  const [amplitude, setAmplitude] = useState(0);
  const [playing, setPlaying] = useState(false);
  const animationRef = useRef<number | null>(null);

  // Desaturate the color for background and fader border
  const desaturatedColor = useMemo(() => {
    return color ? desaturateColor(color, 0.25, 0.4) : undefined;
  }, [color, volume]);

  useEffect(() => {
    const handleStart = () => setPlaying(true);
    const handleStop = () => setPlaying(false);

    transport.on("start", handleStart);
    transport.on("stop", handleStop);
    transport.on("pause", handleStop);

    return () => {
      transport.removeListener("start", handleStart);
      transport.removeListener("stop", handleStop);
      transport.removeListener("pause", handleStop);
    };
  }, []);

  useEffect(() => {
    if (!playing) {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
      return;
    }

    const updateAmplitude = () => {
      setAmplitude(sampler.RMS(instrument.id.toString()));
      animationRef.current = requestAnimationFrame(updateAmplitude);
    };

    updateAmplitude();

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [playing, instrument.id]);
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
          amplitude={amplitude}
          color={desaturatedColor}
          onChange={(val) => onVolumeChange(Math.round((val / 100) * 127))}
        />
      </div>

      {/* Solo and Mute buttons */}
      <div className="flex items-center gap-2 p-2">
        <IconButton
          path={mdiSizeS}
          variant="outline"
          size={0.9}
          isSelected={isSoloed}
          onClick={onToggleSolo}
        />
        <IconButton
          path={mdiSizeM}
          variant="outline"
          size={0.9}
          isSelected={isMuted}
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
