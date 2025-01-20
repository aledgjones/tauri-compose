import { chromeDark, ObjectInspector } from "react-inspector";
import { useAppState } from "../../services/state";
import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppBar } from "../../components/app-bar";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import classNames from "classnames";
import { Log } from "../../services/types";
import { Button } from "../../ui/button";
import { actions } from "../../services/actions";

export const Main = () => {
  const state = useAppState();
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [logs, setLogs] = useState<Log[]>([]);

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<Log>("solocompose://logging", (event) => {
        setLogs((logs) => [...logs.slice(-49), event.payload]);
      });
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_2fr_1fr] gap-2 overflow-hidden">
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
      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        {logs.map((log, i) => {
          return (
            <div key={i}>
              <pre
                className={classNames("font-mono text-[11px]", {
                  "text-white": log.type === "log",
                  "text-red-500": log.type === "error",
                  "text-yellow-500": log.type === "warn",
                })}
              >
                {log.message}
              </pre>
            </div>
          );
        })}
        <div className="h-0" ref={messagesEndRef} />
      </div>
      <div className="overflow-auto mx-2 mb-2">
        <Button
          onClick={async () => {
            actions.reset();
          }}
        >
          Reset
        </Button>
      </div>
    </div>
  );
};
