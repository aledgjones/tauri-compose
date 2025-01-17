import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { createContext, FunctionComponent } from "preact";
import { useContext, useEffect, useState } from "preact/hooks";
import { AppState, AutoCountStyle, Tab } from "./types";
import { actions } from "./actions";

export const initialAppState: AppState = {
  ui: {
    tab: Tab.Setup,
    layout_id: "full-score",
    setup: { selection: null, expanded: [] },
  },
  score: {
    layouts: { order: [], by_id: {} },
    flows: { order: [], by_id: {} },
    players: { order: [], by_id: {} },
    instruments: { by_id: {} },
    tracks: { by_id: {} },
    config: {
      auto_count_styles: {
        solo: AutoCountStyle.Roman,
        section: AutoCountStyle.Roman,
      },
    },
    meta: {
      app_version: "0.1.0",
      title: "",
      subtitle: "",
      dedication: "",
      composer: "",
      arranger: "",
      lyricist: "",
      artist: "",
      copyist: "",
      publisher: "",
      editor: "",
      copyright: "",
      work_number: "",
      composition_year: "",
      composer_dates: "",
      other_information: "",
      created: Date.now(),
    },
  },
};

export const AppContext = createContext<AppState | null>(null);

export const AppStateProvider: FunctionComponent = ({ children }) => {
  const [appState, setAppState] = useState<AppState>(initialAppState);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<AppState>("solocompose://state", (e) => {
        setAppState(e.payload);
      });

      await actions.initialize();
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
