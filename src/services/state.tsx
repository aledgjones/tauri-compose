import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { createContext, FunctionComponent } from "preact";
import { useContext, useEffect, useState } from "preact/hooks";

export enum Tab {
  Setup = 0,
  Write = 1,
  Engrave = 2,
  Sequence = 3,
  Print = 4,
}

export type AppState = {
  ui: { tab: Tab; layout_id: string };
  score: {
    meta: {
      app_version: string;
      title: string;
      subtitle: string;
      composer: string;
      arranger: string;
      lyricist: string;
      copyright: string;
      created: number;
    };
  };
};

export const initialAppState: AppState = {
  ui: { tab: Tab.Setup, layout_id: "full-score" },
  score: {
    meta: {
      app_version: "0.1.0",
      title: "",
      subtitle: "",
      composer: "",
      arranger: "",
      lyricist: "",
      copyright: "",
      created: 1735592276909,
    },
  },
};

export const AppContext = createContext<AppState | null>(null);

export const actions = {
  ui: {
    setTab: (tab: Tab) => {
      invoke("set_ui_tab", { tab });
    },
    setActiveLayoutId: (layout_id: string) => {
      invoke("set_ui_layout_id", { layout_id });
    },
  },
  score: {
    meta: {
      setTitle: (title: string) => {
        invoke("set_score_meta_title", { title });
      },
      setSubtitle: (subtitle: string) => {
        invoke("set_score_meta_subtitle", { subtitle });
      },
      setComposer: (composer: string) => {
        invoke("set_score_meta_composer", { composer });
      },
      setArranger: (arranger: string) => {
        invoke("set_score_meta_arranger", { arranger });
      },
      setLyricist: (lyricist: string) => {
        invoke("set_score_meta_lyricist", { lyricist });
      },
      setCopyright: (copyright: string) => {
        invoke("set_score_meta_copyright", { copyright });
      },
    },
  },
};

export const AppStateProvider: FunctionComponent = ({ children }) => {
  const [appState, setAppState] = useState<AppState>(initialAppState);

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
