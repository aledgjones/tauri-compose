import { useAppState, useSamplerState } from "../../services/state";
import { useCallback, useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { AppBar } from "../../components/app-bar";
import { Input } from "../../ui/input";
import { Select, SelectOption } from "../../ui/select";
import { SnapPicker } from "../../components/snap-picker";
import { actions } from "../../services/actions";
import {
  parseBarlinePopover,
  parseClefPopover,
  parseKeySignaturePopover,
  parseTimeSignaturePopover,
  PopoverType,
} from "../../services/popovers";
import { Flow, UniqueId } from "../../services/types";
import { chromeDark, ObjectInspector } from "react-inspector";

const POPOVERS: SelectOption<PopoverType>[] = [
  { value: "barline", label: "Barline" },
  { value: "clef", label: "Clef" },
  { value: "dynamic", label: "Dynamic" },
  { value: "key-signature", label: "Key Signature" },
  { value: "measure", label: "Measure" },
];

export const Main = () => {
  const state = useAppState();
  const samplerState = useSamplerState();

  const flowId = state.score.flows.order[0] || null;
  const flow: Flow | null = (flowId && state.score.flows.by_id[flowId]) || null;
  const subdivisions = flow?.subdivisions || 256;
  const ticksPerSnap = (subdivisions * 4) / state.ui.sequencer.snap;

  const [tick, setTick] = useState(0);
  const [popoverType, setPopoverType] = useState<PopoverType>("barline");
  const [popover, setPopover] = useState("");
  const [instrumentId, setInstrumentId] = useState<UniqueId>(-1);
  const [staveId, setStaveId] = useState<UniqueId>(-1);
  const [renderTimings, setRenderTimings] = useState<{
    total_ms: number;
    parsing_ms: number;
    drawing_ms: number;
  } | null>(null);

  useEffect(() => {
    const unlisten = listen("render-timings", (event) => {
      setRenderTimings(event.payload as any);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const instruments: SelectOption<UniqueId>[] = useMemo(() => {
    return state.score.players.order.flatMap((playerId) => {
      const player = state.score.players.by_id[playerId];
      return player.instrument_ids.map((instrumentId) => {
        const instrument = state.score.instruments.by_id[instrumentId];
        return {
          value: instrumentId,
          label:
            instrument.long_name +
            (instrument.count ? ` ${instrument.count}` : ""),
        };
      });
    });
  }, [state]);

  const staves = useMemo(() => {
    if (instrumentId !== -1) {
      return state.score.instruments.by_id[instrumentId].stave_ids.map(
        (id, index) => {
          return { label: `Stave ${index + 1}`, value: id };
        },
      );
    }

    return [];
  }, [instrumentId, state.score.instruments.by_id]);

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  const handleEnter = useCallback(async () => {
    if (!flowId) {
      return;
    }

    switch (popoverType) {
      case "barline": {
        try {
          parseBarlinePopover(popover, flowId, tick);
          setPopover("");
        } catch (error) {
          console.log(error);
        }
        break;
      }
      case "clef": {
        try {
          parseClefPopover(
            popover,
            flowId,
            tick,
            staveId > -1 ? staveId : undefined,
          );
          setPopover("");
        } catch (error) {
          console.log(error);
        }
        break;
      }
      case "key-signature": {
        try {
          parseKeySignaturePopover(popover, flowId, tick);
          setPopover("");
        } catch (error) {
          console.log(error);
        }
        break;
      }
      case "measure": {
        try {
          parseTimeSignaturePopover(popover, flowId, tick);
          setPopover("");
        } catch (error) {
          console.log(error);
        }
        break;
      }

      default:
        break;
    }
  }, [flowId, popover, popoverType, staveId, tick]);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_auto_1fr_2fr] gap-2 overflow-hidden">
      <AppBar title="Debugger" filename={state.score.meta.title} />

      <div className="bg-neutral-900 p-2 mx-2 rounded">
        <label className="text-xs text-neutral-400">Render Timings</label>
        <pre className="text-white text-xs mb-2">
          total: {renderTimings?.total_ms || 0} ms | parsing:{" "}
          {renderTimings?.parsing_ms || 0} ms | drawing:{" "}
          {renderTimings?.drawing_ms || 0} ms
        </pre>
      </div>

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={{
            ui: state.ui,
            score: state.score,
            sampler: samplerState,
            next_id: state.next_id,
          }}
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
        <div className="flex gap-2">
          <SnapPicker
            label="Snap"
            value={state.ui.sequencer.snap}
            onChange={actions.ui.sequencer.setSnap}
          />
          <Input
            min={0}
            className="w-18.75"
            type="number"
            label="Tick"
            value={tick}
            onChange={setTick}
            step={ticksPerSnap}
            decimalPlaces={0}
          />

          <Select
            className="flex-1"
            label="Instrument"
            variant="filled"
            value={instrumentId}
            options={instruments}
            onChange={setInstrumentId}
          />
          <Select
            className="flex-1"
            label="Voice"
            variant="filled"
            value={staveId}
            options={staves}
            onChange={setStaveId}
          />
        </div>

        <div className="flex gap-2 mb-2">
          <Select
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

        {popoverType === "barline" && (
          <div className="text-white text-xs p-2 bg-neutral-800 flex-1">
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Clear At Tick</pre>
              <pre className="">"clear"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Normal (Single)</pre>
              <pre className="">"|", "single", "normal"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Double</pre>
              <pre className="">"||", "double"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Triple</pre>
              <pre className="">"|||", "triple"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Dashed</pre>
              <pre className="">":", "dash", "dashed"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Final</pre>
              <pre className="">"|]", "final"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Start Repeat</pre>
              <pre className="">"|:", "start"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">End Repeat</pre>
              <pre className="">":|", "end"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">End/Start Repeat</pre>
              <pre className="">":|:", ":||:", "end-start", "endstart"</pre>
            </div>
          </div>
        )}

        {popoverType === "clef" && (
          <div className="text-white text-xs p-2 bg-neutral-800 flex-1">
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Clear At Tick</pre>
              <pre className="">"clear"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Treble</pre>
              <pre className="">"treble", "G", "g"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Alto</pre>
              <pre className="">"alto", "CA", "ca"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Tenor</pre>
              <pre className="">"tenor", "CT", "ct"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Bass</pre>
              <pre className="">"bass", "F", "f"</pre>
            </div>
          </div>
        )}

        {popoverType === "key-signature" && (
          <div className="text-white text-xs p-2 bg-neutral-800 flex-1">
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Clear At Tick</pre>
              <pre className="">"clear"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Open/Atonal</pre>
              <pre className="">"open", "atonal"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Major Keys (uppercase)</pre>
              <pre className="">"C", "G", "G#", "Ab" etc.</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Minor Keys (lowercase)</pre>
              <pre className="">"c", "g", "g#", "ab" etc.</pre>
            </div>
          </div>
        )}

        {popoverType === "measure" && (
          <div className="text-white text-xs p-2 bg-neutral-800 flex-1">
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Clear At Tick</pre>
              <pre className="">"clear"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Simple time signatures</pre>
              <pre className="">"2/4", "6/8", "3/4", "5/4"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Common Time</pre>
              <pre className="">"c"</pre>
            </div>
            <div className="flex">
              <pre className="pr-8 font-bold w-50">Cut Common Time</pre>
              <pre className="">"cutc"</pre>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
