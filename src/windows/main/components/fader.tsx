import { FC, useCallback, useRef } from "react";

interface FaderProps {
  volume: number;
  onChange: (value: number) => void;
  disabled?: boolean;
  amplitude?: number;
  color?: string;
}

export const Fader: FC<FaderProps> = ({
  volume,
  onChange,
  disabled = false,
  amplitude = 0,
  color = "#9ca3af",
}) => {
  const trackRef = useRef<HTMLDivElement>(null);

  const handlePointerDown = useCallback(
    (e: React.PointerEvent<HTMLButtonElement>) => {
      if (disabled || !trackRef.current) return;

      e.preventDefault();
      const track = trackRef.current;
      const rect = track.getBoundingClientRect();

      const updateVolume = (clientY: number) => {
        const offset = rect.bottom - clientY;
        const newVolume = Math.max(
          0,
          Math.min(100, (offset / rect.height) * 100),
        );
        onChange(Math.round(newVolume));
      };

      updateVolume(e.clientY);

      const handlePointerMove = (e: PointerEvent) => {
        updateVolume(e.clientY);
      };

      const handlePointerUp = () => {
        document.removeEventListener("pointermove", handlePointerMove);
        document.removeEventListener("pointerup", handlePointerUp);
      };

      document.addEventListener("pointermove", handlePointerMove);
      document.addEventListener("pointerup", handlePointerUp);
    },
    [onChange, disabled],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLButtonElement>) => {
      if (disabled) return;

      if (e.key === "ArrowUp") {
        e.preventDefault();
        onChange(Math.min(100, volume + 1));
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        onChange(Math.max(0, volume - 1));
      }
    },
    [volume, onChange, disabled],
  );

  return (
    <div className="relative h-full w-0">
      {/* Fader handle */}
      <button
        className="absolute left-0 w-5 h-9 rounded cursor-pointer z-10 flex items-center justify-center"
        style={{
          bottom: `${volume}%`,
          backgroundColor: disabled ? "#4b5563" : "#ccc",
          transform: "translate(-50%, 50%)",
        }}
        onPointerDown={handlePointerDown}
        onKeyDown={handleKeyDown}
        disabled={disabled}
      >
        <div className="w-3 h-0.5 bg-neutral-400" />
      </button>

      {/* Scale markings */}
      {Array.from({ length: 11 }, (_, i) => (
        <div
          key={i}
          className="absolute h-0.5 z-0 bg-neutral-900"
          style={{
            top: `${i * 10}%`,
            left: "50%",
            transform: `translate(-50%, ${i * -10}%)`,
            width: i % 5 === 0 ? "calc(100% + 34px)" : "calc(100% + 24px)",
          }}
        />
      ))}

      {/* Track */}
      <div
        ref={trackRef}
        className="relative h-full w-1.5 transform -translate-x-1/2 overflow-hidden z-0 bg-neutral-900"
        style={{ outline: `4px solid ${color}` }}
      >
        {/* Amplitude meter */}
        <div
          className="absolute left-0 bottom-0 w-full"
          style={{
            height: `${amplitude * 100}%`,
            backgroundColor: color,
          }}
        />
      </div>
    </div>
  );
};
