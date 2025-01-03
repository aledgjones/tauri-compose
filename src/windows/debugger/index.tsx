import { chromeDark, Inspector } from "react-inspector";
import { useAppState } from "../../services/state";
import { useEffect, useState } from "preact/hooks";
import { window } from "@tauri-apps/api";
import { AppBar } from "../../ui/app-bar";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export type Log = {
  type: "log" | "error" | "warn";
  message: string;
};

export const Main = () => {
  const state = useAppState();
  const [logs, setLogs] = useState<Log[]>([]);

  useEffect(() => {
    const show = async () => {
      const win = window.getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<Log>("solocompose://logging", (event) => {
        setLogs((logs) => [...logs, event.payload]);
      });
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 flex flex-col">
      <AppBar title="Debugger" filename={state.score.meta.title} />
      <div className="grid grid-rows-2 flex-1 p-2 gap-2">
        <div className="rounded-lg bg-neutral-900 overflow-hidden h-full p-2">
          <Inspector
            data={state}
            theme={{ ...chromeDark, BASE_BACKGROUND_COLOR: "transparent" }}
            expandPaths={["$"]}
          />
        </div>
        <div className="rounded-lg bg-neutral-900 overflow-hidden h-full p-2">
          {logs.map((log, i) => {
            return (
              <div key={i} className="flex items-center">
                <p className="font-mono text-sm">{log}</p>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
