import { FC, useCallback, useRef } from "react";
import { mdiSizeM, mdiSizeS } from "@mdi/js";
import { IconButton } from "../../../ui/buttons/icon-button";
import { Fader } from "./fader";
import { Instrument } from "../../../services/types";
import { TokenizedText } from "../../../ui/text";

const PAN_CENTER = 64;
const PAN_MIN = 0;
const PAN_MAX = 127;

const clampPan = (value: number) => {
  return Math.max(PAN_MIN, Math.min(PAN_MAX, Math.round(value)));
};

const panOffsetLabel = (pan: number) => {
  const offset = pan - PAN_CENTER;
  if (offset === 0) {
    return "C";
  }

  if (offset < 0) {
    return `L${Math.abs(offset)}`;
  }

  return `R${offset}`;
};

const panDisplayLabel = (pan: number) => {
  return panOffsetLabel(pan);
};

type MixerProps = {
  instrument: Instrument;
  disabled?: boolean;
  color?: string;
  showPanControl?: boolean;
  showVolumeControl?: boolean;
  showMuteControl?: boolean;
  showSoloControl?: boolean;
  pan: number;
  volume: number;
  isMuted: boolean;
  isSoloed: boolean;
  onPanChange: (pan: number) => void;
  onVolumeChange: (volume: number) => void;
  onToggleMute: () => void;
  onToggleSolo: () => void;
};

export const MixerChannel: FC<MixerProps> = ({
  instrument,
  disabled = false,
  color,
  showPanControl = false,
  showVolumeControl = true,
  showMuteControl = false,
  showSoloControl = false,
  pan,
  volume,
  isMuted,
  isSoloed,
  onPanChange,
  onVolumeChange,
  onToggleMute,
  onToggleSolo,
}) => {
  const panControlRef = useRef<HTMLButtonElement>(null);

  const updatePanFromClientX = useCallback(
    (clientX: number) => {
      const control = panControlRef.current;
      if (!control || disabled) {
        return;
      }

      const rect = control.getBoundingClientRect();
      const ratio = rect.width <= 0 ? 0.5 : (clientX - rect.left) / rect.width;
      onPanChange(clampPan(ratio * PAN_MAX));
    },
    [disabled, onPanChange],
  );

  const handlePanPointerDown = useCallback(
    (event: React.PointerEvent<HTMLButtonElement>) => {
      if (disabled) {
        return;
      }

      event.preventDefault();
      updatePanFromClientX(event.clientX);

      const handlePointerMove = (moveEvent: PointerEvent) => {
        updatePanFromClientX(moveEvent.clientX);
      };

      const handlePointerUp = () => {
        document.removeEventListener("pointermove", handlePointerMove);
        document.removeEventListener("pointerup", handlePointerUp);
      };

      document.addEventListener("pointermove", handlePointerMove);
      document.addEventListener("pointerup", handlePointerUp);
    },
    [disabled, updatePanFromClientX],
  );

  const handlePanKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLButtonElement>) => {
      if (disabled) {
        return;
      }

      if (event.key === "ArrowLeft") {
        event.preventDefault();
        onPanChange(clampPan(pan - 1));
      } else if (event.key === "ArrowRight") {
        event.preventDefault();
        onPanChange(clampPan(pan + 1));
      } else if (event.key === "Home") {
        event.preventDefault();
        onPanChange(PAN_CENTER);
      }
    },
    [disabled, onPanChange, pan],
  );

  const barStartPercent = pan >= PAN_CENTER ? 50 : (pan / PAN_MAX) * 100;
  const barWidthPercent = (Math.abs(pan - PAN_CENTER) / PAN_CENTER) * 50;

  return (
    <div className="flex flex-col w-24 h-full bg-neutral-900">
      {showPanControl && (
        <div className="p-2 pb-0">
          <button
            ref={panControlRef}
            type="button"
            role="slider"
            aria-label={`Pan ${instrument.short_name}`}
            aria-valuemin={PAN_MIN}
            aria-valuemax={PAN_MAX}
            aria-valuenow={pan}
            aria-valuetext={panDisplayLabel(pan)}
            className="relative w-full overflow-hidden bg-neutral-950 p-2 text-center text-xs text-sky-500 outline-none"
            disabled={disabled}
            onPointerDown={handlePanPointerDown}
            onDoubleClick={() => onPanChange(PAN_CENTER)}
            onKeyDown={handlePanKeyDown}
          >
            <div className="absolute inset-0 bg-neutral-950" />
            {barWidthPercent > 0 && (
              <div
                className="absolute bottom-0 top-0 bg-sky-600"
                style={{
                  left: `${barStartPercent}%`,
                  width: `${barWidthPercent}%`,
                }}
              />
            )}
            <div
              className="absolute bottom-0 top-0 w-px bg-sky-500/60"
              style={{ left: "50%" }}
            />
            <div className="relative z-10 block leading-none text-white text-shadow-xs">
              {panDisplayLabel(pan)}
            </div>
          </button>
        </div>
      )}

      <div className="flex flex-1">
        {(showSoloControl || showMuteControl) && (
          <div className="flex flex-col gap-2 pl-2 py-2">
            {showSoloControl && (
              <IconButton
                path={mdiSizeS}
                variant="filled-dark"
                size={0.9}
                isSelected={isSoloed}
                onClick={onToggleSolo}
                aria-label={`Solo ${instrument.short_name}`}
                title={`Solo ${instrument.short_name}`}
              />
            )}
            {showMuteControl && (
              <IconButton
                path={mdiSizeM}
                variant="filled-dark"
                size={0.9}
                isSelected={isMuted}
                onClick={onToggleMute}
                aria-label={`Mute ${instrument.short_name}`}
                title={`Mute ${instrument.short_name}`}
              />
            )}
          </div>
        )}

        <div className="flex flex-1 items-center justify-center px-2 py-4">
          {showVolumeControl && (
            <Fader
              volume={volume}
              disabled={disabled}
              amplitude={0}
              color={color}
              onChange={(val) => onVolumeChange(Math.round((val / 100) * 127))}
            />
          )}
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
