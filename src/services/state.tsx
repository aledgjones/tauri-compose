import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { createContext, FunctionComponent } from "preact";
import { useContext, useEffect, useState } from "preact/hooks";

export enum Tab {
  Setup = 0,
  Write = 1,
  Engrave = 2,
  Play = 3,
  Print = 4,
}

type AppState = { ui: { tab: Tab } };

export const AppContext = createContext<AppState | null>(null);

export const actions = {
  ui: {
    setTab: (tab: Tab) => invoke("ui_set_tab", { tab }),
  },
};

export const AppStateProvider: FunctionComponent = ({ children }) => {
  const [appState, setAppState] = useState<AppState>({
    ui: { tab: Tab.Setup },
  });

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<AppState>("solocompose://state", (e) => {
        console.log(e);
        setAppState(e.payload);
      });

      await invoke("initialize");
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  return <AppContext.Provider value={appState}>{children}</AppContext.Provider>;
};

export const useAppState = () => {
  const context = useContext(AppContext);

  if (!context) {
    throw new Error("useAppState must be used within an AppStateProvider");
  }

  return context;
};
