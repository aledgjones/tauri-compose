import { useAppState, useAudioState, useVstState } from "../../services/state";
import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppBar } from "../../components/app-bar";
import { actions } from "../../services/actions";
import { chromeDark, ObjectInspector } from "react-inspector";
import { Switch } from "../../ui/switch";
import { Button } from "../../ui/button";
import { Select } from "../../ui/select";
import { PerformanceTimings } from "./components/performance-timings";

export const Main = () => {
  const state = useAppState();
  const audio = useAudioState();
  const vst = useVstState();
  const [selectedPlugin, setSelectedPlugin] = useState<string>("");

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  const handleLoadPlugin = async () => {
    if (!selectedPlugin) return;

    try {
      await actions.vst.loadPlugin(selectedPlugin);
    } catch (error) {
      console.error("Failed to load plugin:", error);
    }
  };

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_auto_auto_auto_1fr_1fr_1fr_auto] gap-2 overflow-hidden">
      <AppBar title="Debugger" />

      <PerformanceTimings />

      <div className="bg-neutral-900 p-2 mx-2 flex items-center">
        <Switch
          label="Render Spacing Debugging"
          value={state.ui.renderer.debug}
          onChange={() => actions.ui.renderer.toggleDebug()}
        />
      </div>

      <div className="bg-neutral-900 p-2 mx-2">
        <div className="flex items-center gap-2">
          <Select
            variant="filled"
            value={selectedPlugin}
            onChange={setSelectedPlugin}
            options={Object.entries(vst.plugins || {}).map(([id, plugin]) => ({
              value: id,
              label: plugin.name,
            }))}
            placeholder="Select a VST plugin..."
          />
          <Button onClick={handleLoadPlugin} isDisabled={!selectedPlugin}>
            Load Plugin
          </Button>
        </div>
      </div>

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={{
            ui: state.ui,
            score: state.score,
            preferences: state.preferences,
            current_file_path: state.current_file_path,
          }}
          name="state"
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
          name="audio"
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
          data={vst}
          name="vst"
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
