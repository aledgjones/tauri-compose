import {
  useAppState,
  useAudioState,
  useSamplerState,
} from "../../services/state";
import { useEffect, useMemo } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppBar } from "../../components/app-bar";
import { actions } from "../../services/actions";
import { chromeDark, ObjectInspector } from "react-inspector";
import { Switch } from "../../ui/switch";
import { PerformanceTimings } from "./components/performance-timings";

export const Main = () => {
  const state = useAppState();
  const audio = useAudioState();
  const sampler = useSamplerState();
  const pluginAuditionState = useMemo(() => {
    return {
      ...sampler,
      plugins: sampler.plugins.map(({ path: _path, ...plugin }) => plugin),
    };
  }, [sampler]);

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_auto_auto_1fr_1fr_1fr] gap-2 overflow-hidden pb-2">
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
        <div className="mb-2 text-xs font-semibold uppercase tracking-[0.18em] text-neutral-500">
          Legacy Plugin Audition State
        </div>
        <ObjectInspector
          data={pluginAuditionState}
          name="plugin_audition_state"
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
