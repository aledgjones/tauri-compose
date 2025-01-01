import { FunctionalComponent, render } from "preact";
import { useEffect, useState } from "preact/hooks";
import { AppState, initialAppState, Tab } from "../services/state";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { Inspector } from "react-inspector";

const Debug: FunctionalComponent = () => {
  const [appState, setAppState] = useState<AppState>(initialAppState);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<AppState>("solocompose://state", (e) => {
        setAppState(e.payload);
      });

      await invoke("initialize");
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  return <Inspector data={appState} />;
};

render(<Debug />, document.getElementById("root")!);
