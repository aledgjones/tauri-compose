import { chromeDark, ObjectInspector } from "react-inspector";
import { useAppState } from "../../services/state";
import { useCallback, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppBar } from "../../components/app-bar";
import { Input } from "../../ui/input";
import { Select, SelectOption } from "../../ui/select";
import { SnapPicker } from "../../components/snap-picker";
import { actions } from "../../services/actions";
import { parseBarlinePopover, PopoverType } from "../../services/popovers";
import { Flow } from "../../services/types";

const POPOVERS: SelectOption<PopoverType>[] = [
  { value: "barline", label: "Barline" },
  { value: "clef", label: "Clef" },
  { value: "dynamic", label: "Dynamic" },
  { value: "measure", label: "Measure" },
];

export const Main = () => {
  const state = useAppState();

  const flowId = state.score.flows.order[0];
  const flow: Flow | null = state.score.flows.by_id[flowId] || null;
  const subdivisions = flow?.subdivisions || 256;
  const ticksPerSnap = (subdivisions * 4) / state.ui.sequencer.snap;

  const [tick, setTick] = useState(0);
  const [popoverType, setPopoverType] = useState<PopoverType>("barline");
  const [popover, setPopover] = useState("");

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  const handleEnter = useCallback(async () => {
    switch (popoverType) {
      case "barline":
        try {
          parseBarlinePopover(popover, flowId, tick);
          setPopover("");
        } catch (error) {
          console.log(error);
        }
        break;

      default:
        break;
    }
  }, [flowId, popover, popoverType, tick]);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr_1fr] gap-2 overflow-hidden">
      <AppBar title="Debugger" filename={state.score.meta.title} />

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={state}
          theme={
            {
              ...chromeDark,
              BASE_BACKGROUND_COLOR: "transparent",
            } as unknown as string
          }
          expandPaths={["$"]}
        />
      </div>

      <div className="bg-neutral-900 p-2 pt-0 mx-2 z-50 mb-2 flex flex-col">
        <div className="flex gap-2 mb-2">
          <SnapPicker
            label="Snap"
            value={state.ui.sequencer.snap}
            onChange={actions.ui.sequencer.setSnap}
          />
          <Input
            min={0}
            className="w-[75px]"
            type="number"
            label="Tick"
            value={tick}
            onChange={setTick}
            step={ticksPerSnap}
            decimalPlaces={0}
          />
          <Select
            className="w-[120px]"
            variant="filled"
            label="Popover Type"
            options={POPOVERS}
            value={popoverType}
            onChange={setPopoverType}
          />
          <Input
            type="text"
            className="flex-1"
            label="Popover"
            value={popover}
            onChange={setPopover}
            onEnter={handleEnter}
          />
        </div>

        <div className="text-white text-xs p-2 bg-neutral-800 flex-1">
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">Clear At Tick</pre>
            <pre className="">"clear"</pre>
          </div>
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">Normal (Single)</pre>
            <pre className="">"|", "single", "normal"</pre>
          </div>
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">Double</pre>
            <pre className="">"||", "double"</pre>
          </div>
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">Triple</pre>
            <pre className="">"|||", "triple"</pre>
          </div>
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">Dashed</pre>
            <pre className="">":", "dash", "dashed"</pre>
          </div>
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">Final</pre>
            <pre className="">"|]", "final"</pre>
          </div>
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">Start Repeat</pre>
            <pre className="">"|:", "start"</pre>
          </div>
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">End Repeat</pre>
            <pre className="">":|", "end"</pre>
          </div>
          <div className="flex">
            <pre className="pr-8 font-bold w-[200px]">End/Start Repeat</pre>
            <pre className="">":|:", ":||:", "end-start", "endstart"</pre>
          </div>
        </div>
      </div>
    </div>
  );
};
