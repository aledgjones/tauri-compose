import MdiIcon from "@mdi/react";
import { mdiPlus, mdiMinus, mdiMagnify } from "@mdi/js";
import { Select } from "../../../ui/select";
import { useAppState } from "../../../services/state";
import { actions } from "../../../services/actions";

const zoomOptions = [
  { value: "25", label: "25%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "50", label: "50%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "75", label: "75%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "100", label: "100%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "125", label: "125%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "150", label: "150%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "200", label: "200%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "300", label: "300%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "400", label: "400%", icon: mdiMagnify, iconSize: 0.75 },
  { value: "800", label: "800%", icon: mdiMagnify, iconSize: 0.75 },
];

export const ZoomWidget = () => {
  const state = useAppState();
  const currentZoom = state.ui.renderer.zoom.toString();

  const handleZoomChange = (zoom: string) => {
    actions.ui.renderer.setZoom(parseInt(zoom));
  };

  const handleZoomIncrement = () => {
    const currentIndex = zoomOptions.findIndex(
      (option) => option.value === currentZoom,
    );
    if (currentIndex < zoomOptions.length - 1) {
      const nextZoom = zoomOptions[currentIndex + 1].value;
      actions.ui.renderer.setZoom(parseInt(nextZoom));
    }
  };

  const handleZoomDecrement = () => {
    const currentIndex = zoomOptions.findIndex(
      (option) => option.value === currentZoom,
    );
    if (currentIndex > 0) {
      const prevZoom = zoomOptions[currentIndex - 1].value;
      actions.ui.renderer.setZoom(parseInt(prevZoom));
    }
  };

  return (
    <div className="flex items-center">
      <Select
        value={currentZoom}
        options={zoomOptions}
        onChange={handleZoomChange}
        variant="standard"
        className="[&>button]:h-7 [&>button]:px-1 [&>button]:text-xs [&>button]:border-0 [&>button]:bg-neutral-900 [&>button]:hover:bg-neutral-800 [&>ul]:mt-1"
        direction="up"
      />
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
