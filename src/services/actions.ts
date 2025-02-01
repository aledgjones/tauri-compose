import { invoke } from "@tauri-apps/api/core";
import {
  Accidental,
  Articulation,
  AutoCountStyle,
  BracketingApproach,
  BracketStyle,
  LayoutType,
  NoteDuration,
  PlayerType,
  SelectionType,
  Tab,
  Tick,
  Ticks,
  Tool,
  UniqueId,
} from "./types";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export const actions = {
  initialize: () => {
    return invoke("initialize");
  },
  reset: () => {
    return invoke("reset");
  },
  render: (layout_id: UniqueId, flow_id: UniqueId, px_per_mm: number) => {
    return invoke<ArrayBuffer>("render", {
      layout_id,
      flow_id,
      px_per_mm,
    });
  },
  pickAnInstrument: () => {
    return new Promise<string | null>(async (resolve) => {
      console.log("[instrument-picker] open");

      const webview = new WebviewWindow("instrument-picker", {
        url: "/instrument-picker.html",
        title: "Instrument Picker",
        visible: false,
        titleBarStyle: "overlay",
        acceptFirstMouse: true,
        height: 500,
        width: 900,
        theme: "dark",
      });

      let selection: string | null = null;

      const blurUnlisten = await webview.once("tauri://blur", async () => {
        console.log("[instrument-picker] blur");
        webview.close();
      });

      const selectInstrumentUnlisten = await webview.once<{
        instrument_id: string;
      }>("solocompose://select-instrument", async (e) => {
        console.log(
          `[instrument-picker] select-instrument - payload ${JSON.stringify(
            e.payload
          )}`
        );

        selection = e.payload?.instrument_id;
        webview.close();
      });

      await webview.once("tauri://destroyed", async () => {
        console.log("[instrument-picker] close");
        blurUnlisten();
        selectInstrumentUnlisten();
        resolve(selection);
      });
    });
  },
  ui: {
    setTab: (tab: Tab) => {
      return invoke("ui_tab_set", { tab });
    },
    setActiveLayoutId: (layout_id: number) => {
      return invoke("ui_layout_id_set", { layout_id });
    },
    setup: {
      select: (selection_type: SelectionType, id: number) => {
        return invoke("ui_setup_selection_set", { selection_type, id });
      },
      clearSelection: () => {
        return invoke("ui_setup_selection_clear");
      },
      expand: (id: number) => {
        return invoke("ui_setup_expanded_insert", { id });
      },
      collapse: (id: number) => {
        return invoke("ui_setup_expanded_remove", { id });
      },
    },
    sequencer: {
      setInstrument: (id: UniqueId) => {
        return invoke("ui_sequencer_instrument_set", { id });
      },
      setTrack: (id: UniqueId) => {
        return invoke("ui_sequencer_track_set", { id });
      },
      setTool: (tool: Tool) => {
        return invoke("ui_sequencer_tool_set", { tool });
      },
      setSnap: (snap: NoteDuration) => {
        return invoke("ui_sequencer_snap_set", { snap });
      },
      setCoords: (x: number, y: number) => {
        return invoke("ui_sequencer_coords_set", { x, y });
      },
    },
  },
  score: {
    engrave: {
      setSystemicBarlineSingleInstrumentSystem: (
        layout_type: LayoutType,
        value: boolean
      ) => {
        return invoke<void>(
          "score_engraves_systemic_barline_single_instrument_system_set",
          { layout_type, value }
        );
      },
      setBracketingApproach: (
        layout_type: LayoutType,
        value: BracketingApproach
      ) => {
        return invoke<void>("score_engraves_bracketing_approach_set", {
          layout_type,
          value,
        });
      },
      setBracketStyle: (layout_type: LayoutType, value: BracketStyle) => {
        return invoke<void>("score_engraves_bracket_style_set", {
          layout_type,
          value,
        });
      },
      setBracketSingleStaves: (layout_type: LayoutType, value: boolean) => {
        return invoke<void>("score_engraves_bracket_single_staves_set", {
          layout_type,
          value,
        });
      },
    },
    flow: {
      create: () => {
        return invoke<number>("score_flows_create");
      },
      rename: (flow_id: UniqueId, title: string) => {
        return invoke<void>("score_flows_rename", { flow_id, title });
      },
      move: (from_index: number, to_index: number) => {
        return invoke<void>("score_flows_move", { from_index, to_index });
      },
      assignPlayer: (flow_id: UniqueId, player_id: UniqueId) => {
        return invoke<void>("score_flows_assign_player", {
          flow_id,
          player_id,
        });
      },
      unassignPlayer: (flow_id: UniqueId, player_id: UniqueId) => {
        return invoke<void>("score_flows_unassign_player", {
          flow_id,
          player_id,
        });
      },
      remove: (flow_id: UniqueId) => {
        return invoke<void>("score_flows_remove", { flow_id });
      },
    },
    layout: {
      create: (layout_type: LayoutType) => {
        return invoke<number>("score_layouts_create", {
          layout_type,
        });
      },
      rename: (layout_id: UniqueId, title: string) => {
        return invoke<void>("score_layouts_rename", {
          layout_id,
          title,
        });
      },
      assignPlayer: (layout_id: UniqueId, player_id: UniqueId) => {
        return invoke<void>("score_layouts_assign_player", {
          layout_id,
          player_id,
        });
      },
      unassignPlayer: (layout_id: UniqueId, player_id: UniqueId) => {
        return invoke<void>("score_layouts_unassign_player", {
          layout_id,
          player_id,
        });
      },
      assignFlow: (layout_id: UniqueId, player_id: UniqueId) => {
        return invoke<void>("score_layouts_assign_flow", {
          layout_id,
          player_id,
        });
      },
      unassignFlow: (layout_id: UniqueId, player_id: UniqueId) => {
        return invoke<void>("score_layouts_unassign_flow", {
          layout_id,
          player_id,
        });
      },
      move: (from_index: number, to_index: number) => {
        return invoke<void>("score_layouts_move", {
          from_index,
          to_index,
        });
      },
      remove: (layout_id: UniqueId) => {
        return invoke<void>("score_layouts_remove", {
          layout_id,
        });
      },
    },
    player: {
      create: (player_type: PlayerType) => {
        return invoke<number>("score_players_create", {
          player_type,
        });
      },
      assignInstrument: (player_id: number, instrument_id: number) => {
        return invoke<void>("score_players_assign_instrument", {
          player_id,
          instrument_id,
        });
      },
      unassignInstrument: (player_id: number, instrument_id: number) => {
        return invoke<void>("score_players_unassign_instrument", {
          player_id,
          instrument_id,
        });
      },
      move: (from_index: number, to_index: number) => {
        return invoke<void>("score_players_move", {
          from_index,
          to_index,
        });
      },
      remove: (player_id: number) => {
        return invoke<void>("score_players_remove", {
          player_id,
        });
      },
    },
    instrument: {
      create: (definition_id: string) => {
        return invoke<number>("score_instruments_create", {
          definition_id,
        });
      },
      move: (
        from_index: number,
        to_index: number,
        from_player_id: number,
        to_player_id: number
      ) => {
        return invoke<void>("score_instruments_move", {
          from_index,
          to_index,
          from_player_id,
          to_player_id,
        });
      },
      remove: (instrument_id: number) => {
        return invoke<number>("score_instruments_remove", {
          instrument_id,
        });
      },
    },
    config: {
      autoCountStyles: {
        setSolo: (auto_count_style: AutoCountStyle) => {
          return invoke("score_config_auto_count_styles_solo_set", {
            auto_count_style,
          });
        },
        setSection: (auto_count_style: AutoCountStyle) => {
          return invoke("score_config_auto_count_styles_section_set", {
            auto_count_style,
          });
        },
      },
    },
    meta: {
      setTitle: (title: string) => {
        return invoke("score_meta_title_set", { title });
      },
      setSubtitle: (subtitle: string) => {
        return invoke("score_meta_subtitle_set", { subtitle });
      },
      setDedication: (dedication: string) => {
        return invoke("score_meta_dedication_set", { dedication });
      },
      setComposer: (composer: string) => {
        return invoke("score_meta_composer_set", { composer });
      },
      setArranger: (arranger: string) => {
        return invoke("score_meta_arranger_set", { arranger });
      },
      setLyricist: (lyricist: string) => {
        return invoke("score_meta_lyricist_set", { lyricist });
      },
      setArtist: (artist: string) => {
        return invoke("score_meta_artist_set", { artist });
      },
      setCopyist: (copyist: string) => {
        return invoke("score_meta_copyist_set", { copyist });
      },
      setPublisher: (publisher: string) => {
        return invoke("score_meta_publisher_set", { publisher });
      },
      setEditor: (editor: string) => {
        return invoke("score_meta_editor_set", { editor });
      },
      setCopyright: (copyright: string) => {
        return invoke("score_meta_copyright_set", { copyright });
      },
      setWorkNumber: (work_number: string) => {
        return invoke("score_meta_work_number_set", { work_number });
      },
      setCompositionYear: (composition_year: string) => {
        return invoke("score_meta_composition_year_set", {
          composition_year,
        });
      },
      setComposerDates: (composer_dates: string) => {
        return invoke("score_meta_composer_dates_set", {
          composer_dates,
        });
      },
      setOtherInformation: (other_information: string) => {
        return invoke("score_meta_other_information_set", {
          other_information,
        });
      },
    },
    tone: {
      create: (
        track_id: UniqueId,
        tick: Tick,
        duration: Ticks,
        pitch_int: number,
        accidental: Accidental | null,
        velocity: number,
        articulation: Articulation
      ) => {
        return invoke("score_entries_tone_create", {
          track_id,
          tick,
          duration,
          pitch_int,
          accidental,
          velocity,
          articulation,
        });
      },
    },
  },
};
