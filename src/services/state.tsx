import { invoke, InvokeArgs, InvokeOptions } from "@tauri-apps/api/core";
import { emit, listen, UnlistenFn } from "@tauri-apps/api/event";
import { createContext, FunctionComponent } from "preact";
import { useContext, useEffect, useState } from "preact/hooks";
import { Log } from "../windows/debugger";

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
      dedication: string;
      composer: string;
      arranger: string;
      lyricist: string;
      artist: string;
      copyist: string;
      publisher: string;
      editor: string;
      copyright: string;
      work_number: string;
      composition_year: string;
      composer_dates: string;
      other_information: string;
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

export const loggedInvoke = async (
  cmd: string,
  args?: InvokeArgs,
  options?: InvokeOptions
) => {
  console.log(process.env);
  if (process.env.TAURI_DEBUG === "false") {
    return invoke(cmd, args, options);
  }

  actions.console.log(`[invoke] ${cmd} - ${JSON.stringify(args)}`);

  try {
    return invoke(cmd, args, options);
  } catch (error) {
    actions.console.error(`[invoke:error]: ${cmd} - ${JSON.stringify(error)}`);
    throw error;
  }
};

export const actions = {
  console: {
    log: (message: string) => {
      const log: Log = {
        type: "log",
        message,
      };
      emit("solocompose://logging", log);
    },
    warn: (message: string) => {
      const log: Log = {
        type: "warn",
        message,
      };
      emit("solocompose://logging", log);
    },
    error: (message: string) => {
      const log: Log = {
        type: "error",
        message,
      };
      emit("solocompose://logging", log);
    },
  },
  ui: {
    setTab: (tab: Tab) => {
      loggedInvoke("set_ui_tab", { tab });
    },
    setActiveLayoutId: (layout_id: string) => {
      loggedInvoke("set_ui_layout_id", { layout_id });
    },
  },
  score: {
    meta: {
      setTitle: (title: string) => {
        loggedInvoke("set_score_meta_title", { title });
      },
      setSubtitle: (subtitle: string) => {
        loggedInvoke("set_score_meta_subtitle", { subtitle });
      },
      setDedication: (dedication: string) => {
        loggedInvoke("set_score_meta_dedication", { dedication });
      },
      setComposer: (composer: string) => {
        loggedInvoke("set_score_meta_composer", { composer });
      },
      setArranger: (arranger: string) => {
        loggedInvoke("set_score_meta_arranger", { arranger });
      },
      setLyricist: (lyricist: string) => {
        loggedInvoke("set_score_meta_lyricist", { lyricist });
      },
      setArtist: (artist: string) => {
        loggedInvoke("set_score_meta_artist", { artist });
      },
      setCopyist: (copyist: string) => {
        loggedInvoke("set_score_meta_copyist", { copyist });
      },
      setPublisher: (publisher: string) => {
        loggedInvoke("set_score_meta_publisher", { publisher });
      },
      setEditor: (editor: string) => {
        loggedInvoke("set_score_meta_editor", { editor });
      },
      setCopyright: (copyright: string) => {
        loggedInvoke("set_score_meta_copyright", { copyright });
      },
      setWorkNumber: (work_number: string) => {
        loggedInvoke("set_score_meta_work_number", { work_number });
      },
      setCompositionYear: (composition_year: string) => {
        loggedInvoke("set_score_meta_composition_year", { composition_year });
      },
      setComposerDates: (composer_dates: string) => {
        loggedInvoke("set_score_meta_composer_dates", { composer_dates });
      },
      setOtherInformation: (other_information: string) => {
        loggedInvoke("set_score_meta_other_information", { other_information });
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
