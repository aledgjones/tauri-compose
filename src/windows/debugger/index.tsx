import { useAppState, useAudioState } from "../../services/state";
import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppBar } from "../../components/app-bar";
import { actions } from "../../services/actions";
import { chromeDark, ObjectInspector } from "react-inspector";
import { Switch } from "../../ui/switch";
import { PerformanceTimings } from "./components/performance-timings";

const inspectorTheme = {
  ...chromeDark,
  BASE_BACKGROUND_COLOR: "transparent",
} as unknown as string;

export const Main = () => {
  const state = useAppState();
  const audio = useAudioState();

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_auto_auto_1fr_1fr] gap-2 overflow-hidden pb-2">
      <AppBar title="Debugger" />

      <PerformanceTimings />

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
          name="state"
          theme={inspectorTheme}
          expandPaths={["$"]}
        />
      </div>

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={audio}
          name="audio"
          theme={inspectorTheme}
          expandPaths={["$"]}
        />
      </div>
    </div>
  );
};
