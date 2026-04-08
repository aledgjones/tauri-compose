import { Icon as MdiIcon } from "@mdi/react";
import { mdiPlus, mdiMinus, mdiMagnify, mdiChevronDown } from "@mdi/js";
import { useAppState } from "../../../services/state";
import { actions } from "../../../services/actions";
import { useState, useRef, useEffect } from "react";
import classNames from "classnames";

const zoomPresets = [25, 50, 75, 100, 125, 150, 200, 300, 400, 800];

export const ZoomWidget = () => {
  const state = useAppState();
  const [isOpen, setIsOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  const currentZoom = state.ui.renderer.zoom;

  const handleZoomChange = (zoom: number) => {
    actions.ui.renderer.setZoom(zoom);
    setIsOpen(false);
  };

  const handleZoomIncrement = () => {
    const nextPreset = zoomPresets.find((preset) => preset > currentZoom);
    if (nextPreset) {
      actions.ui.renderer.setZoom(nextPreset);
    }
  };

  const handleZoomDecrement = () => {
    const prevPreset = [...zoomPresets]
      .reverse()
      .find((preset) => preset < currentZoom);
    if (prevPreset) {
      actions.ui.renderer.setZoom(prevPreset);
    }
  };

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [isOpen]);

  return (
    <div className="flex items-center relative" ref={menuRef}>
      <div className="relative">
        <button
          className={classNames(
            "flex items-center gap-1 h-7 px-2 text-xs border-0 hover:bg-neutral-800 text-white",
            { "bg-neutral-800": isOpen, "bg-neutral-900": !isOpen },
          )}
          onClick={() => setIsOpen(!isOpen)}
        >
          <MdiIcon size={0.75} path={mdiMagnify} />
          <span className="pr-2 pl-1">{currentZoom}%</span>
          <MdiIcon
            size={0.6}
            path={mdiChevronDown}
            className={classNames(
              "text-neutral-300 transform transition-transform",
              { "rotate-180": isOpen },
            )}
          />
        </button>

        {isOpen && (
          <ul className="whitespace-nowrap py-1 w-full absolute z-50 text-white text-sm shadow-lg bottom-full mb-0.5 bg-neutral-800">
            {zoomPresets.map((preset) => (
              <li key={preset}>
                <button
                  onClick={() => handleZoomChange(preset)}
                  role="option"
                  aria-selected={preset === currentZoom}
                  className="w-full text-left px-3 py-2 hover:bg-neutral-700 cursor-pointer border-none bg-transparent"
                >
                  {preset}%
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <button
        className="ml-0.5 px-2 h-7 bg-neutral-900 hover:bg-neutral-800 text-white"
        onClick={handleZoomIncrement}
      >
        <MdiIcon size={0.75} path={mdiPlus} />
      </button>
      <button
        className="px-2 h-7 bg-neutral-900 hover:bg-neutral-800 text-white rounded-br-xl"
        onClick={handleZoomDecrement}
      >
        <MdiIcon size={0.75} path={mdiMinus} />
      </button>
    </div>
  );
};
