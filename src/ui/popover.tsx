import { ChangeEvent, FC, useEffect, useRef, useState } from "react";
import { actions } from "../services/actions";

export const Popover: FC = () => {
  const [value, setValue] = useState("");
  const [coords, setCoords] = useState<{ x: number; y: number } | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (coords) {
        return;
      }

      if (e.shiftKey && e.key === "K") {
        e.preventDefault();

        const coords = await actions.ui.renderer.getSelectionCoordinates();
        if (coords) {
          const [x, y] = coords;
          setCoords({ x, y });
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown);

    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [coords]);

  useEffect(() => {
    if (coords) {
      // Focus the input when popover appears
      setTimeout(() => {
        inputRef.current?.focus();
        inputRef.current?.select();
      }, 0);
    }
  }, [coords]);

  if (!coords) {
    return null;
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      setCoords(null);
      setValue("");
    } else if (e.key === "Escape") {
      setCoords(null);
      setValue("");
    }
  };

  const handleChange = (e: ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setValue(newValue);
  };

  const handleBlur = () => {
    setCoords(null);
    setValue("");
  };

  return (
    <div
      className="absolute z-50 pointer-events-auto"
      style={{ left: coords.x, top: coords.y }}
    >
      <div className="shadow-md absolute bottom-0 text-[9px] bg-sky-500 rounded">
        <div className="px-1 py-px text-white">Key Signature</div>
        <input
          ref={inputRef}
          type="text"
          value={value}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          onBlur={handleBlur}
          className="px-2 py-1 text-sm bg-white rounded focus:outline-none border-sky-500 border-2 w-45"
        />
      </div>
    </div>
  );
};
