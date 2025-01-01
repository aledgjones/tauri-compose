import { chromeDark, Inspector } from "react-inspector";
import { useAppState } from "../../services/state";
import { useEffect } from "preact/hooks";
import { window } from "@tauri-apps/api";
import { AppBar } from "../../ui/app-bar";

export const Main = () => {
  const state = useAppState();

  useEffect(() => {
    const show = async () => {
      const win = window.getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 flex flex-col">
      <AppBar title="Debugger" filename={state.score.meta.title} />
      <div className="flex-1 p-2">
        <div className="bg-neutral-900 rounded-lg overflow-hidden h-full p-2">
          <Inspector
            data={state}
            theme={{ ...chromeDark, BASE_BACKGROUND_COLOR: "transparent" }}
            expandPaths={["$"]}
          />
        </div>
      </div>
    </div>
  );
};
