import { useAppState, useSamplerState } from "../../services/state";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { AppBar } from "../../components/app-bar";
import { Select, SelectOption } from "../../ui/select";
import { actions } from "../../services/actions";
import { PopoverType } from "../../services/popovers";
import { chromeDark, ObjectInspector } from "react-inspector";
import classNames from "classnames";
import { Switch } from "../../ui/switch";

const POPOVERS: SelectOption<PopoverType>[] = [
  { value: PopoverType.Barline, label: "Barline" },
  { value: PopoverType.Clef, label: "Clef" },
  { value: PopoverType.KeySignature, label: "Key Signature" },
  { value: PopoverType.TimeSignature, label: "Time Signature" },
];

export const Main = () => {
  const state = useAppState();
  const samplerState = useSamplerState();
  const [popoverType, setPopoverType] = useState<PopoverType>(
    PopoverType.Barline,
  );
  const [renderTimings, setRenderTimings] = useState<Record<string, number>>(
    {},
  );
  const [layoutCollapsed, setLayoutCollapsed] = useState(true);
  const [renderCollapsed, setRenderCollapsed] = useState(true);
  const [gpuCollapsed, setGpuCollapsed] = useState(true);

  useEffect(() => {
    const timingKeys = [
      "render-timing-vertical",
      "render-timing-layout",
      "render-timing-structure",
      "render-timing-notation",
      "render-timing-elements",
      "render-timing-spacing",
      "render-timing-stems",
      "render-timing-setup",
      "render-timing-barlines",
      "render-timing-stave",
      "native-render-timing-setup",
      "native-render-timing-draw",
      "native-render-timing-flush",
    ];

    const unlisteners = timingKeys.map((eventName) =>
      listen(eventName, (event) => {
        const duration = event.payload as number;
        setRenderTimings((prev) => ({ ...prev, [eventName]: duration }));
      }),
    );

    return () => {
      unlisteners.forEach((unlisten) => {
        unlisten.then((fn) => fn());
      });
    };
  }, []);

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_auto_auto_1fr_auto] gap-2 overflow-hidden">
      <AppBar title="Debugger" />

      <div className="bg-neutral-900 p-2 mx-2">
        <button
          className="text-sm font-bold text-neutral-200 bg-neutral-800 px-2 py-1 flex justify-between cursor-pointer hover:bg-neutral-700 w-full text-left border-none"
          onClick={() => setLayoutCollapsed(!layoutCollapsed)}
          type="button"
          aria-expanded={!layoutCollapsed}
        >
          <span>🎼 Layout Phase (CPU)</span>
          <span
            className={classNames("font-mono text-sm", {
              "text-red-400 font-bold":
                ((renderTimings["render-timing-vertical"] || 0) +
                  (renderTimings["render-timing-layout"] || 0) +
                  (renderTimings["render-timing-structure"] || 0) +
                  (renderTimings["render-timing-notation"] || 0) +
                  (renderTimings["render-timing-elements"] || 0) +
                  (renderTimings["render-timing-spacing"] || 0) +
                  (renderTimings["render-timing-stems"] || 0)) /
                  1000 >
                50,
              "text-yellow-400 font-bold":
                ((renderTimings["render-timing-vertical"] || 0) +
                  (renderTimings["render-timing-layout"] || 0) +
                  (renderTimings["render-timing-structure"] || 0) +
                  (renderTimings["render-timing-notation"] || 0) +
                  (renderTimings["render-timing-elements"] || 0) +
                  (renderTimings["render-timing-spacing"] || 0) +
                  (renderTimings["render-timing-stems"] || 0)) /
                  1000 >
                  20 &&
                ((renderTimings["render-timing-vertical"] || 0) +
                  (renderTimings["render-timing-layout"] || 0) +
                  (renderTimings["render-timing-structure"] || 0) +
                  (renderTimings["render-timing-notation"] || 0) +
                  (renderTimings["render-timing-elements"] || 0) +
                  (renderTimings["render-timing-spacing"] || 0) +
                  (renderTimings["render-timing-stems"] || 0)) /
                  1000 <=
                  50,
              "text-green-400 font-bold":
                ((renderTimings["render-timing-vertical"] || 0) +
                  (renderTimings["render-timing-layout"] || 0) +
                  (renderTimings["render-timing-structure"] || 0) +
                  (renderTimings["render-timing-notation"] || 0) +
                  (renderTimings["render-timing-elements"] || 0) +
                  (renderTimings["render-timing-spacing"] || 0) +
                  (renderTimings["render-timing-stems"] || 0)) /
                  1000 <=
                20,
            })}
          >
            {(
              (renderTimings["render-timing-vertical"] || 0) +
              (renderTimings["render-timing-layout"] || 0) +
              (renderTimings["render-timing-structure"] || 0) +
              (renderTimings["render-timing-notation"] || 0) +
              (renderTimings["render-timing-elements"] || 0) +
              (renderTimings["render-timing-spacing"] || 0) +
              (renderTimings["render-timing-stems"] || 0)
            ).toLocaleString()}
            μs
          </span>
        </button>
        {!layoutCollapsed && (
          <div className="px-2 py-1.5 bg-neutral-800">
            <div className="flex items-center">
              <span className="text-xs text-neutral-400 flex-1">
                Vertical spacing & staves
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-vertical"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-vertical"] || 0) / 1000 >
                      20 &&
                    (renderTimings["render-timing-vertical"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-vertical"] || 0) / 1000 <= 20,
                })}
              >
                {(
                  renderTimings["render-timing-vertical"] || 0
                ).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                Layout calculations
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-layout"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-layout"] || 0) / 1000 > 20 &&
                    (renderTimings["render-timing-layout"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-layout"] || 0) / 1000 <= 20,
                })}
              >
                {(renderTimings["render-timing-layout"] || 0).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                Score structure parsing
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-structure"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-structure"] || 0) / 1000 >
                      20 &&
                    (renderTimings["render-timing-structure"] || 0) / 1000 <=
                      50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-structure"] || 0) / 1000 <=
                    20,
                })}
              >
                {(
                  renderTimings["render-timing-structure"] || 0
                ).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                Written durations analysis
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-notation"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-notation"] || 0) / 1000 >
                      20 &&
                    (renderTimings["render-timing-notation"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-notation"] || 0) / 1000 <= 20,
                })}
              >
                {(
                  renderTimings["render-timing-notation"] || 0
                ).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                Musical elements analysis
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-elements"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-elements"] || 0) / 1000 >
                      20 &&
                    (renderTimings["render-timing-elements"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-elements"] || 0) / 1000 <= 20,
                })}
              >
                {(
                  renderTimings["render-timing-elements"] || 0
                ).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                Horizontal spacing
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-spacing"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-spacing"] || 0) / 1000 > 20 &&
                    (renderTimings["render-timing-spacing"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-spacing"] || 0) / 1000 <= 20,
                })}
              >
                {(renderTimings["render-timing-spacing"] || 0).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                Stem length calculations
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-stems"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-stems"] || 0) / 1000 > 20 &&
                    (renderTimings["render-timing-stems"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-stems"] || 0) / 1000 <= 20,
                })}
              >
                {(renderTimings["render-timing-stems"] || 0).toLocaleString()}μs
              </span>
            </div>
          </div>
        )}

        <button
          className="text-sm font-bold text-neutral-200 bg-neutral-800 px-2 py-1 flex justify-between cursor-pointer hover:bg-neutral-700 w-full text-left border-none"
          onClick={() => setRenderCollapsed(!renderCollapsed)}
          type="button"
          aria-expanded={!renderCollapsed}
        >
          <span>✏️ Render Phase (CPU)</span>
          <span
            className={classNames("font-mono text-sm", {
              "text-red-400 font-bold":
                ((renderTimings["render-timing-setup"] || 0) +
                  (renderTimings["render-timing-barlines"] || 0) +
                  (renderTimings["render-timing-stave"] || 0)) /
                  1000 >
                50,
              "text-yellow-400 font-bold":
                ((renderTimings["render-timing-setup"] || 0) +
                  (renderTimings["render-timing-barlines"] || 0) +
                  (renderTimings["render-timing-stave"] || 0)) /
                  1000 >
                  20 &&
                ((renderTimings["render-timing-setup"] || 0) +
                  (renderTimings["render-timing-barlines"] || 0) +
                  (renderTimings["render-timing-stave"] || 0)) /
                  1000 <=
                  50,
              "text-green-400 font-bold":
                ((renderTimings["render-timing-setup"] || 0) +
                  (renderTimings["render-timing-barlines"] || 0) +
                  (renderTimings["render-timing-stave"] || 0)) /
                  1000 <=
                20,
            })}
          >
            {(
              (renderTimings["render-timing-setup"] || 0) +
              (renderTimings["render-timing-barlines"] || 0) +
              (renderTimings["render-timing-stave"] || 0)
            ).toLocaleString()}
            μs
          </span>
        </button>
        {!renderCollapsed && (
          <div className="px-2 py-1.5 bg-neutral-800">
            <div className="flex items-center">
              <span className="text-xs text-neutral-400 flex-1">
                Basic drawing setup
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-setup"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-setup"] || 0) / 1000 > 20 &&
                    (renderTimings["render-timing-setup"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-setup"] || 0) / 1000 <= 20,
                })}
              >
                {(renderTimings["render-timing-setup"] || 0).toLocaleString()}μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                Barline rendering
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-barlines"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-barlines"] || 0) / 1000 >
                      20 &&
                    (renderTimings["render-timing-barlines"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-barlines"] || 0) / 1000 <= 20,
                })}
              >
                {(
                  renderTimings["render-timing-barlines"] || 0
                ).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                Stave contents rendering
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["render-timing-stave"] || 0) / 1000 > 50,
                  "text-yellow-400 font-bold":
                    (renderTimings["render-timing-stave"] || 0) / 1000 > 20 &&
                    (renderTimings["render-timing-stave"] || 0) / 1000 <= 50,
                  "text-green-400 font-bold":
                    (renderTimings["render-timing-stave"] || 0) / 1000 <= 20,
                })}
              >
                {(renderTimings["render-timing-stave"] || 0).toLocaleString()}μs
              </span>
            </div>
          </div>
        )}

        <button
          className="text-sm font-bold text-neutral-200 bg-neutral-800 px-2 py-1 flex justify-between cursor-pointer hover:bg-neutral-700 w-full text-left border-none"
          onClick={() => setGpuCollapsed(!gpuCollapsed)}
          type="button"
          aria-expanded={!gpuCollapsed}
        >
          <span>🚀 GPU Phase (Metal/Skia)</span>
          <span
            className={classNames("font-mono text-sm", {
              "text-red-400 font-bold":
                ((renderTimings["native-render-timing-setup"] || 0) +
                  (renderTimings["native-render-timing-draw"] || 0) +
                  (renderTimings["native-render-timing-flush"] || 0)) /
                  1000 >
                50,
              "text-yellow-400 font-bold":
                ((renderTimings["native-render-timing-setup"] || 0) +
                  (renderTimings["native-render-timing-draw"] || 0) +
                  (renderTimings["native-render-timing-flush"] || 0)) /
                  1000 >
                  20 &&
                ((renderTimings["native-render-timing-setup"] || 0) +
                  (renderTimings["native-render-timing-draw"] || 0) +
                  (renderTimings["native-render-timing-flush"] || 0)) /
                  1000 <=
                  50,
              "text-green-400 font-bold":
                ((renderTimings["native-render-timing-setup"] || 0) +
                  (renderTimings["native-render-timing-draw"] || 0) +
                  (renderTimings["native-render-timing-flush"] || 0)) /
                  1000 <=
                20,
            })}
          >
            {(
              (renderTimings["native-render-timing-setup"] || 0) +
              (renderTimings["native-render-timing-draw"] || 0) +
              (renderTimings["native-render-timing-flush"] || 0)
            ).toLocaleString()}
            μs
          </span>
        </button>
        {!gpuCollapsed && (
          <div className="px-2 py-1.5 bg-neutral-800">
            <div className="flex items-center">
              <span className="text-xs text-neutral-400 flex-1">
                GPU surface & canvas setup
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["native-render-timing-setup"] || 0) / 1000 >
                    50,
                  "text-yellow-400 font-bold":
                    (renderTimings["native-render-timing-setup"] || 0) / 1000 >
                      20 &&
                    (renderTimings["native-render-timing-setup"] || 0) / 1000 <=
                      50,
                  "text-green-400 font-bold":
                    (renderTimings["native-render-timing-setup"] || 0) / 1000 <=
                    20,
                })}
              >
                {(
                  renderTimings["native-render-timing-setup"] || 0
                ).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                GPU drawing execution
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["native-render-timing-draw"] || 0) / 1000 >
                    50,
                  "text-yellow-400 font-bold":
                    (renderTimings["native-render-timing-draw"] || 0) / 1000 >
                      20 &&
                    (renderTimings["native-render-timing-draw"] || 0) / 1000 <=
                      50,
                  "text-green-400 font-bold":
                    (renderTimings["native-render-timing-draw"] || 0) / 1000 <=
                    20,
                })}
              >
                {(
                  renderTimings["native-render-timing-draw"] || 0
                ).toLocaleString()}
                μs
              </span>
            </div>
            <div className="flex items-center bg-neutral-800">
              <span className="text-xs text-neutral-400 flex-1">
                GPU flush & Metal commands
              </span>
              <span
                className={classNames("text-xs font-mono text-right", {
                  "text-red-400 font-bold":
                    (renderTimings["native-render-timing-flush"] || 0) / 1000 >
                    50,
                  "text-yellow-400 font-bold":
                    (renderTimings["native-render-timing-flush"] || 0) / 1000 >
                      20 &&
                    (renderTimings["native-render-timing-flush"] || 0) / 1000 <=
                      50,
                  "text-green-400 font-bold":
                    (renderTimings["native-render-timing-flush"] || 0) / 1000 <=
                    20,
                })}
              >
                {(
                  renderTimings["native-render-timing-flush"] || 0
                ).toLocaleString()}
                μs
              </span>
            </div>
          </div>
        )}
      </div>

      <div className="bg-neutral-900 p-2 mx-2 flex items-center">
        <Switch
          label="Render Spacing Debugging"
          value={state.ui.renderer.debug_rendering}
          onChange={() => actions.ui.renderer.toggleDebugRendering()}
        />
      </div>

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={{
            ui: state.ui,
            score: state.score,
            sampler: samplerState,
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

      <div className="bg-neutral-900 p-2 mx-2 z-50 mb-2 flex flex-col gap-2">
        <Select
          className="flex-1"
          direction="up"
          variant="filled"
          options={POPOVERS}
          value={popoverType}
          onChange={setPopoverType}
        />

        {popoverType === PopoverType.Barline && (
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

        {popoverType === PopoverType.Clef && (
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

        {popoverType === PopoverType.KeySignature && (
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

        {popoverType === PopoverType.TimeSignature && (
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
