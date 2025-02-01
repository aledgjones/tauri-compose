import { chromeDark, ObjectInspector } from "react-inspector";
import { useAppState } from "../../services/state";
import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppBar } from "../../components/app-bar";

export const Main = () => {
  const state = useAppState();

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] gap-2 overflow-hidden">
      <AppBar title="Debugger" filename={state.score.meta.title} />
      <div className="bg-neutral-900 p-2 overflow-auto mx-2 mb-2">
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
    </div>
  );
};
