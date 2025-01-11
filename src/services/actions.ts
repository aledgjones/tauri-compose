import { invoke, InvokeArgs, InvokeOptions } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { AutoCountStyle, Log, PlayerType, SelectionType, Tab } from "./types";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export const loggedInvoke = async <T = unknown>(
  cmd: string,
  args?: InvokeArgs,
  options?: InvokeOptions
) => {
  const t1 = performance.now();

  try {
    const result = await invoke<T>(cmd, args, options);
    actions.console.log(
      `[invoke] ${cmd} - ${JSON.stringify(args)} - ${performance.now() - t1}ms`
    );
    return result;
  } catch (error) {
    actions.console.error(
      `[invoke:error]: ${cmd} - ${JSON.stringify(error)} - ${
        performance.now() - t1
      }ms`
    );
    throw error;
  }
};

export const actions = {
  pickAnInstrument: () => {
    return new Promise<string | null>(async (resolve) => {
      actions.console.log("[instrument-picker] open");

      const webview = new WebviewWindow("instrument-picker", {
        url: "/instrument-picker.html",
        title: "Instrument Picker",
        visible: false,
        titleBarStyle: "overlay",
        hiddenTitle: true,
        acceptFirstMouse: true,
        height: 500,
        width: 900,
      });

      let selection: string | null = null;

      const blurUnlisten = await webview.once("tauri://blur", async () => {
        actions.console.log("[instrument-picker] blur");
        webview.close();
      });

      const selectInstrumentUnlisten = await webview.once<{
        instrument_id: string;
      }>("solocompose://select-instrument", async (e) => {
        actions.console.log(
          `[instrument-picker] select-instrument - payload ${JSON.stringify(
            e.payload
          )}`
        );

        selection = e.payload?.instrument_id;
        webview.close();
      });

      await webview.once("tauri://destroyed", async () => {
        actions.console.log("[instrument-picker] close");
        blurUnlisten();
        selectInstrumentUnlisten();
        resolve(selection);
      });
    });
  },
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
    player: {
      create: (player_type: PlayerType) => {
        return loggedInvoke<number>("create_player", {
          player_type,
        });
      },
      assignInstrument: (player_id: number, instrument_id: number) => {
        return loggedInvoke<void>("assign_instrument_to_player", {
          player_id,
          instrument_id,
        });
      },
      move: (from_index: number, to_index: number) => {
        return loggedInvoke<void>("move_player", {
          from_index,
          to_index,
        });
      },
    },
    instrument: {
      create: (definition_id: string) => {
        return loggedInvoke<number>("create_instrument", {
          definition_id,
        });
      },
      move: (
        from_index: number,
        to_index: number,
        from_player_id: number,
        to_player_id: number
      ) => {
        console.log({ from_index, to_index, from_player_id, to_player_id });
        // return loggedInvoke<void>("move_instrument", {
        //   from_index,
        //   to_index,
        //   from_player_id,
        //   to_player_id,
        // });
      },
    },
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
