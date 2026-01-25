import { useAppState, useAudioState } from "../../services/state";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { AppBar } from "../../components/app-bar";
import { actions } from "../../services/actions";
import { chromeDark, ObjectInspector } from "react-inspector";
import classNames from "classnames";
import { Switch } from "../../ui/switch";

export const Main = () => {
  const state = useAppState();
  const audio = useAudioState();

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
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_auto_auto_1fr_1fr] gap-2 overflow-hidden">
      <AppBar title="Debugger" />

      <div className="mx-2">
        <button
          className="text-sm font-bold text-neutral-200 bg-neutral-900 p-2 flex justify-between cursor-pointer hover:bg-neutral-800 w-full text-left border-none"
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
          <div className="px-2 py-1.5">
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
            <div className="flex items-center">
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
            <div className="flex items-center">
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
            <div className="flex items-center">
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
            <div className="flex items-center">
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
            <div className="flex items-center">
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
            <div className="flex items-center">
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
          className="text-sm font-bold text-neutral-200 bg-neutral-900 p-2 flex justify-between cursor-pointer hover:bg-neutral-800 w-full text-left border-none"
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
          <div className="px-2 py-1.5">
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
            <div className="flex items-center">
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
            <div className="flex items-center">
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
          className="text-sm font-bold text-neutral-200 bg-neutral-900 p-2 flex justify-between cursor-pointer hover:bg-neutral-800 w-full text-left border-none"
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
          <div className="px-2 py-1.5">
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
            <div className="flex items-center">
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
            <div className="flex items-center">
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
          value={state.ui.renderer.debug}
          onChange={() => actions.ui.renderer.toggleDebug()}
        />
      </div>

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={{
            ui: state.ui,
            score: state.score,
            preferences: state.preferences,
            current_file_path: state.current_file_path,
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

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={audio}
          theme={
            {
              ...chromeDark,
              BASE_BACKGROUND_COLOR: "transparent",
            } as unknown as string
          }
          expandPaths={["$"]}
        />
      </div>
    </div>
  );
};
