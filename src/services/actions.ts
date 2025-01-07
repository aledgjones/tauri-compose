import { invoke, InvokeArgs, InvokeOptions } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { AutoCountStyle, Log, PlayerType, SelectionType, Tab } from "./types";

export const loggedInvoke = async <T = unknown>(
  cmd: string,
  args?: InvokeArgs,
  options?: InvokeOptions
) => {
  actions.console.log(`[invoke] ${cmd} - ${JSON.stringify(args)}`);

  try {
    return invoke<T>(cmd, args, options);
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
      return loggedInvoke("set_ui_tab", { tab });
    },
    setActiveLayoutId: (layout_id: string) => {
      return loggedInvoke("set_ui_layout_id", { layout_id });
    },
    setup: {
      setSelection: (selection_type: SelectionType, id: number) => {
        return loggedInvoke("set_ui_setup_selection", { selection_type, id });
      },
      clearSelection: () => {
        return loggedInvoke("clear_ui_setup_selection");
      },
      setExpanded: (id: number) => {
        return loggedInvoke("insert_ui_setup_expanded", { id });
      },
      clearExpanded: (id: number) => {
        return loggedInvoke("remove_ui_setup_expanded", { id });
      },
    },
  },
  score: {
    createPlayer: (player_type: PlayerType) => {
      return loggedInvoke<number>("create_player", {
        player_type,
      });
    },
    createInstrument: () => {},
    config: {
      autoCountStyles: {
        setSolo: (auto_count_style: AutoCountStyle) => {
          return loggedInvoke("set_score_config_auto_count_styles_solo", {
            auto_count_style,
          });
        },
        setSection: (auto_count_style: AutoCountStyle) => {
          return loggedInvoke("set_score_config_auto_count_styles_section", {
            auto_count_style,
          });
        },
      },
    },
    meta: {
      setTitle: (title: string) => {
        return loggedInvoke("set_score_meta_title", { title });
      },
      setSubtitle: (subtitle: string) => {
        return loggedInvoke("set_score_meta_subtitle", { subtitle });
      },
      setDedication: (dedication: string) => {
        return loggedInvoke("set_score_meta_dedication", { dedication });
      },
      setComposer: (composer: string) => {
        return loggedInvoke("set_score_meta_composer", { composer });
      },
      setArranger: (arranger: string) => {
        return loggedInvoke("set_score_meta_arranger", { arranger });
      },
      setLyricist: (lyricist: string) => {
        return loggedInvoke("set_score_meta_lyricist", { lyricist });
      },
      setArtist: (artist: string) => {
        return loggedInvoke("set_score_meta_artist", { artist });
      },
      setCopyist: (copyist: string) => {
        return loggedInvoke("set_score_meta_copyist", { copyist });
      },
      setPublisher: (publisher: string) => {
        return loggedInvoke("set_score_meta_publisher", { publisher });
      },
      setEditor: (editor: string) => {
        return loggedInvoke("set_score_meta_editor", { editor });
      },
      setCopyright: (copyright: string) => {
        return loggedInvoke("set_score_meta_copyright", { copyright });
      },
      setWorkNumber: (work_number: string) => {
        return loggedInvoke("set_score_meta_work_number", { work_number });
      },
      setCompositionYear: (composition_year: string) => {
        return loggedInvoke("set_score_meta_composition_year", {
          composition_year,
        });
      },
      setComposerDates: (composer_dates: string) => {
        return loggedInvoke("set_score_meta_composer_dates", {
          composer_dates,
        });
      },
      setOtherInformation: (other_information: string) => {
        return loggedInvoke("set_score_meta_other_information", {
          other_information,
        });
      },
    },
  },
};
