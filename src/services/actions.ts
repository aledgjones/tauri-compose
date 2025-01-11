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
      return loggedInvoke("ui_tab_set", { tab });
    },
    setActiveLayoutId: (layout_id: string) => {
      return loggedInvoke("ui_layout_id_set", { layout_id });
    },
    setup: {
      select: (selection_type: SelectionType, id: number) => {
        return loggedInvoke("ui_setup_selection_set", { selection_type, id });
      },
      clearSelection: () => {
        return loggedInvoke("ui_setup_selection_clear");
      },
      expand: (id: number) => {
        return loggedInvoke("ui_setup_expanded_insert", { id });
      },
      collapse: (id: number) => {
        return loggedInvoke("ui_setup_expanded_remove", { id });
      },
    },
  },
  score: {
    player: {
      create: (player_type: PlayerType) => {
        return loggedInvoke<number>("score_players_create", {
          player_type,
        });
      },
      assignInstrument: (player_id: number, instrument_id: number) => {
        return loggedInvoke<void>("score_players_assign_instrument", {
          player_id,
          instrument_id,
        });
      },
      unassignInstrument: (player_id: number, instrument_id: number) => {
        return loggedInvoke<void>("score_players_unassign_instrument", {
          player_id,
          instrument_id,
        });
      },
      move: (from_index: number, to_index: number) => {
        return loggedInvoke<void>("score_players_move", {
          from_index,
          to_index,
        });
      },
      remove: (player_id: number) => {
        return loggedInvoke<void>("score_players_remove", {
          player_id,
        });
      },
    },
    instrument: {
      create: (definition_id: string) => {
        return loggedInvoke<number>("score_instruments_create", {
          definition_id,
        });
      },
      move: (
        from_index: number,
        to_index: number,
        from_player_id: number,
        to_player_id: number
      ) => {
        return loggedInvoke<void>("score_instruments_move", {
          from_index,
          to_index,
          from_player_id,
          to_player_id,
        });
      },
      remove: (instrument_id: number) => {
        return loggedInvoke<number>("score_instruments_remove", {
          instrument_id,
        });
      },
    },
    config: {
      autoCountStyles: {
        setSolo: (auto_count_style: AutoCountStyle) => {
          return loggedInvoke("score_config_auto_count_styles_solo_set", {
            auto_count_style,
          });
        },
        setSection: (auto_count_style: AutoCountStyle) => {
          return loggedInvoke("score_config_auto_count_styles_section_set", {
            auto_count_style,
          });
        },
      },
    },
    meta: {
      setTitle: (title: string) => {
        return loggedInvoke("score_meta_title_set", { title });
      },
      setSubtitle: (subtitle: string) => {
        return loggedInvoke("score_meta_subtitle_set", { subtitle });
      },
      setDedication: (dedication: string) => {
        return loggedInvoke("score_meta_dedication_set", { dedication });
      },
      setComposer: (composer: string) => {
        return loggedInvoke("score_meta_composer_set", { composer });
      },
      setArranger: (arranger: string) => {
        return loggedInvoke("score_meta_arranger_set", { arranger });
      },
      setLyricist: (lyricist: string) => {
        return loggedInvoke("score_meta_lyricist_set", { lyricist });
      },
      setArtist: (artist: string) => {
        return loggedInvoke("score_meta_artist_set", { artist });
      },
      setCopyist: (copyist: string) => {
        return loggedInvoke("score_meta_copyist_set", { copyist });
      },
      setPublisher: (publisher: string) => {
        return loggedInvoke("score_meta_publisher_set", { publisher });
      },
      setEditor: (editor: string) => {
        return loggedInvoke("score_meta_editor_set", { editor });
      },
      setCopyright: (copyright: string) => {
        return loggedInvoke("score_meta_copyright_set", { copyright });
      },
      setWorkNumber: (work_number: string) => {
        return loggedInvoke("score_meta_work_number_set", { work_number });
      },
      setCompositionYear: (composition_year: string) => {
        return loggedInvoke("score_meta_composition_year_set", {
          composition_year,
        });
      },
      setComposerDates: (composer_dates: string) => {
        return loggedInvoke("score_meta_composer_dates_set", {
          composer_dates,
        });
      },
      setOtherInformation: (other_information: string) => {
        return loggedInvoke("score_meta_other_information_set", {
          other_information,
        });
      },
    },
  },
};
