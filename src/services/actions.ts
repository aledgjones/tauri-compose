import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import {
  Accidental,
  Articulation,
  AutoCountStyle,
  BarlineDrawType,
  BottomPanelTab,
  BracketingApproach,
  BracketStyle,
  ClefDrawType,
  ExpressionMap,
  ExpressionMapLibraryState,
  KeySignatureMode,
  LayoutType,
  NoteDuration,
  PlayingTechniqueKind,
  PlayerType,
  SetupSelectionType,
  Tab,
  Tick,
  Ticks,
  TimeSignatureDrawType,
  Tool,
  TransportState,
  PlaybackRouteAddress,
  PlaybackRouteTarget,
  UniqueId,
  ExpressionMapExportPayload,
} from "./types";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export const actions = {
  initialize: () => {
    return invoke("initialize");
  },
  reset: () => {
    return invoke("reset");
  },
  openPluginManager: async () => {
    const existing = await WebviewWindow.getByLabel("plugin-manager");

    if (existing) {
      await existing.show();
      await existing.setFocus();

      return existing;
    }

    const webview = new WebviewWindow("plugin-manager", {
      url: "/plugin-manager.html",
      title: "Plugin Manager",
      visible: false,
      hiddenTitle: true,
      titleBarStyle: "overlay",
      acceptFirstMouse: true,
      height: 720,
      width: 1100,
      theme: "dark",
    });

    await webview.show();
    await webview.setFocus();

    return webview;
  },
  openExpressionMapManager: async () => {
    const existing = await WebviewWindow.getByLabel("expression-map");

    if (existing) {
      await existing.emit("soloc://expression-map/select", {
        expression_map_id: "",
      });
      await existing.show();
      await existing.setFocus();

      return existing;
    }

    const webview = new WebviewWindow("expression-map", {
      url: "/expression-map.html",
      title: "Expression Map",
      visible: false,
      hiddenTitle: true,
      titleBarStyle: "overlay",
      acceptFirstMouse: true,
      height: 780,
      width: 1280,
      theme: "dark",
    });

    await webview.show();
    await webview.setFocus();

    return webview;
  },
  openExpressionMapWindow: async (expression_map_id: string) => {
    const existing = await WebviewWindow.getByLabel("expression-map");

    if (existing) {
      await existing.emit("soloc://expression-map/select", {
        expression_map_id,
      });
      await existing.show();
      await existing.setFocus();

      return existing;
    }

    const webview = new WebviewWindow("expression-map", {
      url: `/expression-map.html?expression_map_id=${encodeURIComponent(
        expression_map_id,
      )}`,
      title: "Expression Map",
      visible: false,
      hiddenTitle: true,
      titleBarStyle: "overlay",
      acceptFirstMouse: true,
      height: 780,
      width: 1280,
      theme: "dark",
    });

    await webview.show();
    await webview.setFocus();

    return webview;
  },
  pickAnInstrument: () => {
    return new Promise<string | null>((resolve) => {
      (async () => {
        const webview = new WebviewWindow("instrument-picker", {
          url: "/instrument-picker.html",
          title: "Instrument Picker",
          visible: false,
          hiddenTitle: true,
          titleBarStyle: "overlay",
          acceptFirstMouse: true,
          height: 500,
          width: 900,
          theme: "dark",
        });

        let selection: string | null = null;

        const blurUnlisten = await webview.once("tauri://blur", async () => {
          webview.close();
        });

        const selectInstrumentUnlisten = await webview.once<{
          instrument_id: string;
        }>("soloc://select-instrument", async (e) => {
          selection = e.payload?.instrument_id;
          webview.close();
        });

        await webview.once("tauri://destroyed", async () => {
          blurUnlisten();
          selectInstrumentUnlisten();
          resolve(selection);
        });
      })();
    });
  },
  audio: {
    setDevice: (device_id: string) => {
      return invoke("audio_device_set", { device_id });
    },
    setDeviceSampleRate: (sample_rate: number) => {
      return invoke("audio_device_sample_rate_set", {
        sample_rate,
      });
    },
    refreshDevices: () => invoke("audio_devices_refresh"),
  },
  pluginInventory: {
    discover: () => invoke("plugin_inventory_discover"),
    openEditorPoc: (
      plugin_id: string,
      options?: { plugin_path?: string | null; slot_id?: UniqueId | null },
    ) => {
      return invoke("plugin_editor_poc_open", {
        slot_id: options?.slot_id ?? null,
        plugin_id,
        plugin_path: options?.plugin_path ?? null,
      });
    },
  },
  expressionMaps: {
    getState: () => {
      return invoke<ExpressionMapLibraryState>("expression_maps_get_state");
    },
    create: (name?: string) => {
      return invoke<ExpressionMap>("expression_maps_create", {
        name: name ?? null,
      });
    },
    duplicate: (expression_map_id: string) => {
      return invoke<ExpressionMap>("expression_maps_duplicate", {
        expression_map_id,
      });
    },
    setName: (expression_map_id: string, name: string) => {
      return invoke<ExpressionMap>("expression_maps_name_set", {
        expression_map_id,
        name,
      });
    },
    setCreator: (expression_map_id: string, creator: string) => {
      return invoke<ExpressionMap>("expression_maps_creator_set", {
        expression_map_id,
        creator,
      });
    },
    setVersion: (expression_map_id: string, version: number) => {
      return invoke<ExpressionMap>("expression_maps_version_set", {
        expression_map_id,
        version,
      });
    },
    addRule: (expression_map_id: string) => {
      return invoke<ExpressionMap>("expression_maps_rule_add", {
        expression_map_id,
      });
    },
    setRuleName: (expression_map_id: string, rule_id: string, name: string) => {
      return invoke<ExpressionMap>("expression_maps_rule_name_set", {
        expression_map_id,
        rule_id,
        name,
      });
    },
    moveRule: (
      expression_map_id: string,
      rule_id: string,
      to_index: number,
    ) => {
      return invoke<ExpressionMap>("expression_maps_rule_move", {
        expression_map_id,
        rule_id,
        to_index,
      });
    },
    removeRule: (expression_map_id: string, rule_id: string) => {
      return invoke<ExpressionMap>("expression_maps_rule_remove", {
        expression_map_id,
        rule_id,
      });
    },
    addCondition: (
      expression_map_id: string,
      rule_id: string,
      condition_type: string,
    ) => {
      return invoke<ExpressionMap>("expression_maps_condition_add", {
        expression_map_id,
        rule_id,
        condition_type,
      });
    },
    setConditionType: (
      expression_map_id: string,
      rule_id: string,
      condition_id: string,
      condition_type: string,
    ) => {
      return invoke<ExpressionMap>("expression_maps_condition_set_type", {
        expression_map_id,
        rule_id,
        condition_id,
        condition_type,
      });
    },
    updateCondition: (
      expression_map_id: string,
      rule_id: string,
      condition: ExpressionMap["rules"][number]["conditions"][number],
    ) => {
      return invoke<ExpressionMap>("expression_maps_condition_update", {
        expression_map_id,
        rule_id,
        condition,
      });
    },
    removeCondition: (
      expression_map_id: string,
      rule_id: string,
      condition_id: string,
    ) => {
      return invoke<ExpressionMap>("expression_maps_condition_remove", {
        expression_map_id,
        rule_id,
        condition_id,
      });
    },
    addAction: (
      expression_map_id: string,
      rule_id: string,
      action_type: string,
    ) => {
      return invoke<ExpressionMap>("expression_maps_action_add", {
        expression_map_id,
        rule_id,
        action_type,
      });
    },
    setActionType: (
      expression_map_id: string,
      rule_id: string,
      action_id: string,
      action_type: string,
    ) => {
      return invoke<ExpressionMap>("expression_maps_action_set_type", {
        expression_map_id,
        rule_id,
        action_id,
        action_type,
      });
    },
    updateAction: (
      expression_map_id: string,
      rule_id: string,
      action: ExpressionMap["rules"][number]["actions"][number],
    ) => {
      return invoke<ExpressionMap>("expression_maps_action_update", {
        expression_map_id,
        rule_id,
        action,
      });
    },
    removeAction: (
      expression_map_id: string,
      rule_id: string,
      action_id: string,
    ) => {
      return invoke<ExpressionMap>("expression_maps_action_remove", {
        expression_map_id,
        rule_id,
        action_id,
      });
    },
    addDefaultAction: (expression_map_id: string, action_type: string) => {
      return invoke<ExpressionMap>("expression_maps_default_action_add", {
        expression_map_id,
        action_type,
      });
    },
    setDefaultActionType: (
      expression_map_id: string,
      action_id: string,
      action_type: string,
    ) => {
      return invoke<ExpressionMap>("expression_maps_default_action_set_type", {
        expression_map_id,
        action_id,
        action_type,
      });
    },
    updateDefaultAction: (
      expression_map_id: string,
      action: ExpressionMap["default_actions"][number],
    ) => {
      return invoke<ExpressionMap>("expression_maps_default_action_update", {
        expression_map_id,
        action,
      });
    },
    removeDefaultAction: (expression_map_id: string, action_id: string) => {
      return invoke<ExpressionMap>("expression_maps_default_action_remove", {
        expression_map_id,
        action_id,
      });
    },
    delete: (expression_map_id: string) => {
      return invoke("expression_maps_delete", {
        expression_map_id,
      });
    },
    importFromDialog: async () => {
      const filePath = await open({
        multiple: false,
        title: "Import Expression Maps",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });

      if (filePath == null || Array.isArray(filePath)) {
        return [];
      }

      return invoke<string[]>("expression_maps_import", {
        file_path: filePath,
      });
    },
    exportToDialog: async (expression_map_ids: string[]) => {
      if (expression_map_ids.length === 0) {
        return null;
      }

      const filePath = await save({
        title: "Export Expression Maps",
        defaultPath:
          expression_map_ids.length === 1
            ? `${expression_map_ids[0]}.expression-maps.json`
            : "expression-maps.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });

      if (filePath == null) {
        return null;
      }

      await invoke<ExpressionMapExportPayload>("expression_maps_export", {
        file_path: filePath,
        expression_map_ids,
      });

      return filePath;
    },
  },
  ui: {
    setTab: (tab: Tab) => {
      return invoke("ui_tab_set", { tab });
    },
    setActiveLayoutId: (layout_id: number) => {
      return invoke("ui_layout_id_set", { layout_id });
    },
    setActiveFlowId: (flow_id: number) => {
      return invoke("ui_flow_id_set", { flow_id });
    },
    preferences: {
      setAudition: (audition: boolean) => {
        return invoke("preferences_audition_set", {
          audition,
        });
      },
    },
    renderer: {
      setZoom: (zoom: number) => {
        return invoke<[number, number, number]>("ui_renderer_zoom_set", {
          zoom,
        });
      },
      setPan: (x: number, y: number) => {
        return invoke<[number, number]>("ui_renderer_pan_set", { x, y });
      },
      setBounds: (
        px_per_mm: number,
        bounds: {
          x: number;
          y: number;
          width: number;
          height: number;
        },
      ) => {
        return invoke("ui_renderer_bounds_set", { bounds, px_per_mm });
      },
      toggleDebug: () => {
        return invoke("ui_renderer_debug_toggle");
      },
      getSelectionCoordinates: () => {
        return invoke<[number, number] | null>("ui_get_selection_coordinates");
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
      selectTone: (
        tone_id: number,
        track_id: number,
        stave_id: number,
        flow_id: number,
        instrument_id: number,
        tuplet_id?: number | null,
      ) => {
        return invoke("ui_sequencer_selection_set_tone", {
          tone_id,
          track_id,
          stave_id,
          flow_id,
          instrument_id,
          tuplet_id: tuplet_id ?? null,
        });
      },
      selectRest: (
        tick: number,
        track_id: number,
        stave_id: number,
        flow_id: number,
        instrument_id: number,
        tuplet_id?: number | null,
      ) => {
        return invoke("ui_sequencer_selection_set_rest", {
          tick,
          track_id,
          stave_id,
          flow_id,
          instrument_id,
          tuplet_id: tuplet_id ?? null,
        });
      },
      selectTempo: (tempo_id: number, flow_id: number, tick: number) => {
        return invoke("ui_sequencer_selection_set_tempo", {
          tempo_id,
          flow_id,
          tick,
        });
      },
      selectTimeSignature: (
        flow_id: number,
        time_signature_id: number,
        tick: number,
      ) => {
        return invoke("ui_sequencer_selection_set_time_signature", {
          flow_id,
          time_signature_id,
          tick,
        });
      },
      selectKeySignature: (
        flow_id: number,
        key_signature_id: number,
        tick: number,
      ) => {
        return invoke("ui_sequencer_selection_set_key_signature", {
          flow_id,
          key_signature_id,
          tick,
        });
      },
      selectClef: (
        flow_id: number,
        stave_id: number,
        clef_id: number,
        tick: number,
      ) => {
        return invoke("ui_sequencer_selection_set_clef", {
          flow_id,
          stave_id,
          clef_id,
          tick,
        });
      },
      selectTuplet: (
        flow_id: number,
        instrument_id: number,
        stave_id: number,
        track_id: number,
        tuplet_id: number,
        tick: number,
      ) => {
        return invoke("ui_sequencer_selection_set_tuplet", {
          flow_id,
          instrument_id,
          stave_id,
          track_id,
          tuplet_id,
          tick,
        });
      },
      clearSelection: () => {
        return invoke<void>("ui_sequencer_selection_clear");
      },
    },
    screens: {
      setup: {
        select: (selection_type: SetupSelectionType, id: number) => {
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
      write: {
        bottomPanel: {
          toggle: () => {
            return invoke("ui_write_bottom_panel_minimized_toggle");
          },
          setTab: (tab: BottomPanelTab) => {
            return invoke("ui_write_bottom_panel_tab_set", { tab });
          },
        },
      },
      sequence: {
        bottomPanel: {
          toggle: () => {
            return invoke("ui_sequence_bottom_panel_minimized_toggle");
          },
          setTab: (tab: BottomPanelTab) => {
            return invoke("ui_sequence_bottom_panel_tab_set", { tab });
          },
          instruments: {
            setShowActiveOnly: (show_active_only: boolean) => {
              return invoke("ui_sequence_show_active_only_set", {
                show_active_only,
              });
            },
          },
        },
      },
    },
  },
  score: {
    engrave: {
      setSystemicBarlineSingleInstrumentSystem: (
        layout_type: LayoutType,
        value: boolean,
      ) => {
        return invoke<void>(
          "score_engraves_systemic_barline_single_instrument_system_set",
          { layout_type, value },
        );
      },
      setBracketingApproach: (
        layout_type: LayoutType,
        value: BracketingApproach,
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
      setBaseNoteSpace: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_base_note_space_set", {
          layout_type,
          value,
        });
      },
      setMinimumNoteSpace: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_minimum_note_space_set", {
          layout_type,
          value,
        });
      },
      setMinimumTieSpace: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_minimum_tie_space_set", {
          layout_type,
          value,
        });
      },
      setNoteSpaceRatio: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_note_space_ratio_set", {
          layout_type,
          value,
        });
      },
      setSpace: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_space_set", {
          layout_type,
          value,
        });
      },
      setFramePaddingTop: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_frame_padding_top_set", {
          layout_type,
          value,
        });
      },
      setFramePaddingBottom: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_frame_padding_bottom_set", {
          layout_type,
          value,
        });
      },
      setPagePaddingTop: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_page_padding_top_set", {
          layout_type,
          value,
        });
      },
      setPagePaddingRight: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_page_padding_right_set", {
          layout_type,
          value,
        });
      },
      setPagePaddingBottom: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_page_padding_bottom_set", {
          layout_type,
          value,
        });
      },
      setPagePaddingLeft: (layout_type: LayoutType, value: number) => {
        return invoke<void>("score_engraves_page_padding_left_set", {
          layout_type,
          value,
        });
      },
    },
    flow: {
      create: async () => {
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
      assignFlow: (layout_id: UniqueId, flow_id: UniqueId) => {
        return invoke<void>("score_layouts_assign_flow", {
          layout_id,
          flow_id,
        });
      },
      unassignFlow: (layout_id: UniqueId, flow_id: UniqueId) => {
        return invoke<void>("score_layouts_unassign_flow", {
          layout_id,
          flow_id,
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
        to_player_id: number,
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
      assignSlotRoute: (
        instrument_id: number,
        slot_id: number,
        target: PlaybackRouteTarget,
        address: PlaybackRouteAddress,
      ) => {
        return invoke<void>("score_instruments_assign_slot_route", {
          instrument_id,
          route: {
            slot_id,
            target,
            address,
          },
        });
      },
      unassignRoute: (instrument_id: number) => {
        return invoke<void>("score_instruments_unassign_route", {
          instrument_id,
        });
      },
      setExpressionMap: (
        instrument_id: number,
        expression_map_id: string | null,
      ) => {
        return invoke<void>("score_instruments_set_expression_map", {
          instrument_id,
          expression_map_id,
        });
      },
      setVolume: (instrument_id: number, volume: number) => {
        return invoke<void>("score_instruments_set_volume", {
          instrument_id,
          volume,
        });
      },
      setPan: (instrument_id: number, pan: number) => {
        return invoke<void>("score_instruments_set_pan", {
          instrument_id,
          pan,
        });
      },
      setMute: (instrument_id: number, mute: boolean) => {
        return invoke<void>("score_instruments_set_mute", {
          instrument_id,
          mute,
        });
      },
      setSolo: (instrument_id: number, solo: boolean) => {
        return invoke<void>("score_instruments_set_solo", {
          instrument_id,
          solo,
        });
      },
    },
    playback: {
      slot: {
        create: () => {
          return invoke<number>("score_playback_slots_create");
        },
        remove: (slot_id: number) => {
          return invoke<void>("score_playback_slots_remove", {
            slot_id,
          });
        },
        assignPlugin: (slot_id: number, plugin_id: string) => {
          return invoke<void>("score_playback_slots_assign_plugin", {
            slot_id,
            plugin_id,
          });
        },
        unassignPlugin: (slot_id: number) => {
          return invoke<void>("score_playback_slots_unassign_plugin", {
            slot_id,
          });
        },
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
    barline: {
      create: (
        flow_id: UniqueId,
        tick: Tick,
        barline_draw_type: BarlineDrawType,
      ) => {
        return invoke<UniqueId | null>("score_entries_barline_create", {
          flow_id,
          tick,
          barline_draw_type,
        });
      },
      clearAtTick: (flow_id: UniqueId, tick: Tick) => {
        return invoke("score_entries_barline_clear", {
          flow_id,
          tick,
        });
      },
    },
    clef: {
      create: (
        flow_id: UniqueId,
        tick: Tick,
        stave_id: UniqueId,
        pitch: number,
        offset: number,
        clef_draw_type: ClefDrawType,
        hidden = false,
      ) => {
        return invoke<UniqueId | null>("score_entries_clef_create", {
          flow_id,
          tick,
          stave_id,
          pitch,
          offset,
          clef_draw_type,
          hidden,
        });
      },
      clear: (flow_id: UniqueId, tick: Tick, stave_id: UniqueId) => {
        return invoke<UniqueId | null>("score_entries_clef_clear", {
          flow_id,
          tick,
          stave_id,
        });
      },
      setHidden: (
        flow_id: UniqueId,
        stave_id: UniqueId,
        clef_id: UniqueId,
        hidden: boolean,
      ) => {
        return invoke<void>("score_entries_clef_update", {
          flow_id,
          stave_id,
          clef_id,
          hidden,
        });
      },
    },
    playingTechnique: {
      create: (
        flow_id: UniqueId,
        tick: Tick,
        stave_id: UniqueId,
        kind: PlayingTechniqueKind,
        hidden = false,
      ) => {
        return invoke<UniqueId>("score_entries_playing_technique_create", {
          flow_id,
          tick,
          stave_id,
          kind,
          hidden,
        });
      },
      remove: (
        flow_id: UniqueId,
        stave_id: UniqueId,
        playing_technique_id: UniqueId,
      ) => {
        return invoke<void>("score_entries_playing_technique_remove", {
          flow_id,
          stave_id,
          playing_technique_id,
        });
      },
      update: (
        flow_id: UniqueId,
        stave_id: UniqueId,
        playing_technique_id: UniqueId,
        hidden: boolean,
      ) => {
        return invoke<void>("score_entries_playing_technique_update", {
          flow_id,
          stave_id,
          playing_technique_id,
          hidden,
        });
      },
    },
    keySignature: {
      create: (
        flow_id: UniqueId,
        tick: Tick,
        mode: KeySignatureMode,
        offset: number,
        hidden = false,
      ) => {
        return invoke<UniqueId>("score_entries_key_signature_create", {
          flow_id,
          tick,
          mode,
          offset,
          hidden,
        });
      },
      clearAtTick: (flow_id: UniqueId, tick: Tick) => {
        return invoke("score_entries_key_signature_clear", {
          flow_id,
          tick,
        });
      },
      setHidden: (
        flow_id: UniqueId,
        key_signature_id: UniqueId,
        hidden: boolean,
      ) => {
        return invoke<void>("score_entries_key_signature_update", {
          flow_id,
          key_signature_id,
          hidden,
        });
      },
    },
    timeSignature: {
      create: (
        flow_id: UniqueId,
        tick: Tick,
        beats: number,
        beat_type: NoteDuration,
        draw_type: TimeSignatureDrawType,
        hidden = false,
      ) => {
        return invoke<UniqueId>("score_entries_time_signature_create", {
          flow_id,
          tick,
          beats,
          beat_type,
          draw_type,
          hidden,
        });
      },
      clearAtTick: (flow_id: UniqueId, tick: Tick) => {
        return invoke("score_entries_time_signature_clear", {
          flow_id,
          tick,
        });
      },
      update: (
        flow_id: UniqueId,
        time_signature_id: UniqueId,
        draw_type: TimeSignatureDrawType,
        hidden: boolean,
      ) => {
        return invoke<void>("score_entries_time_signature_update", {
          flow_id,
          time_signature_id,
          draw_type,
          hidden,
        });
      },
    },
    tempo: {
      create: (
        flow_id: UniqueId,
        tick: Tick,
        text: string,
        beat_type: NoteDuration,
        dots: number,
        bpm: number,
        hidden: boolean,
        show_text: boolean,
        show_metronome: boolean,
        parenthesized: boolean,
      ) => {
        return invoke<UniqueId>("score_entries_tempo_create", {
          flow_id,
          tick,
          text,
          beat_type,
          dots,
          bpm,
          hidden,
          show_text,
          show_metronome,
          parenthesized,
        });
      },
      update: (
        flow_id: UniqueId,
        tempo_id: UniqueId,
        text: string,
        beat_type: NoteDuration,
        dots: number,
        bpm: number,
        hidden: boolean,
        show_text: boolean,
        show_metronome: boolean,
        parenthesized: boolean,
      ) => {
        return invoke<void>("score_entries_tempo_update", {
          flow_id,
          tempo_id,
          text,
          beat_type,
          dots,
          bpm,
          hidden,
          show_text,
          show_metronome,
          parenthesized,
        });
      },
      clearAtTick: (flow_id: UniqueId, tick: Tick) => {
        return invoke("score_entries_tempo_clear", {
          flow_id,
          tick,
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
        articulations: Articulation[],
        tuplet_id?: UniqueId,
      ) => {
        return invoke<UniqueId>("score_entries_tone_create", {
          track_id,
          tick,
          duration,
          pitch_int,
          accidental,
          velocity,
          articulations,
          tuplet_id: tuplet_id ?? null,
        });
      },
      shift: (
        track_id: UniqueId,
        entry_id: UniqueId,
        new_tick: Tick,
        current_tuplet_id?: UniqueId,
        target_tuplet_id?: UniqueId,
      ) => {
        return invoke("score_entries_tone_shift", {
          track_id,
          entry_id,
          new_tick,
          current_tuplet_id: current_tuplet_id ?? null,
          target_tuplet_id: target_tuplet_id ?? null,
        });
      },
      unifyArticulations: (
        track_id: UniqueId,
        entry_id: UniqueId,
        tuplet_id?: UniqueId,
      ) => {
        return invoke<void>("score_entries_tone_unify_articulations", {
          track_id,
          entry_id,
          tuplet_id: tuplet_id ?? null,
        });
      },
      setDuration: (
        track_id: UniqueId,
        tone_id: UniqueId,
        duration: Ticks,
        tuplet_id?: UniqueId,
      ) => {
        return invoke("score_entries_tone_duration_set", {
          track_id,
          tone_id,
          duration,
          tuplet_id: tuplet_id ?? null,
        });
      },
      setPositionAndDuration: (
        track_id: UniqueId,
        tone_id: UniqueId,
        tick: Tick,
        duration: Ticks,
        current_tuplet_id?: UniqueId,
        target_tuplet_id?: UniqueId,
      ) => {
        return invoke("score_entries_tone_set_position_and_duration", {
          track_id,
          tone_id,
          tick,
          duration,
          current_tuplet_id: current_tuplet_id ?? null,
          target_tuplet_id: target_tuplet_id ?? null,
        });
      },
      setPitch: (
        track_id: UniqueId,
        tuplet_id: UniqueId | null,
        tone_id: UniqueId,
        pitch_int: number,
        accidental?: Accidental | null,
      ) => {
        return invoke("score_entries_tone_pitch_set", {
          track_id,
          tuplet_id: tuplet_id ?? null,
          tone_id,
          pitch_int,
          accidental,
        });
      },
      remove: (
        track_id: UniqueId,
        tuplet_id: UniqueId | null,
        tone_id: UniqueId,
      ) => {
        return invoke<void>("score_entries_tone_duration_remove", {
          track_id,
          tuplet_id: tuplet_id ?? null,
          tone_id,
        });
      },
      slice: (
        track_id: UniqueId,
        tone_id: UniqueId,
        slice_tick: Tick,
        current_tuplet_id?: UniqueId | null,
        target_tuplet_id?: UniqueId | null,
      ) => {
        return invoke<UniqueId | null>("score_entries_tone_slice", {
          track_id,
          tone_id,
          slice_tick,
          current_tuplet_id: current_tuplet_id ?? null,
          target_tuplet_id: target_tuplet_id ?? null,
        });
      },
      shiftPitchByStep: (
        track_id: UniqueId,
        tuplet_id: UniqueId | null,
        tone_id: UniqueId,
        steps: number,
      ) => {
        return invoke<number | null>(
          "score_entries_tone_pitch_shift_by_steps",
          {
            track_id,
            tuplet_id: tuplet_id ?? null,
            tone_id,
            steps,
          },
        );
      },
      shiftPitchByOctave: (
        track_id: UniqueId,
        tuplet_id: UniqueId | null,
        tone_id: UniqueId,
        octaves: number,
      ) => {
        return invoke<number | null>(
          "score_entries_tone_pitch_shift_by_octaves",
          {
            track_id,
            tuplet_id: tuplet_id ?? null,
            tone_id,
            octaves,
          },
        );
      },
      setAccidental: (
        track_id: UniqueId,
        tuplet_id: UniqueId | null,
        tone_id: UniqueId,
        accidental: Accidental,
      ) => {
        return invoke<number | null>("score_entries_tone_accidental_set", {
          track_id,
          tuplet_id: tuplet_id ?? null,
          tone_id,
          accidental,
        });
      },
      setArticulation: (
        track_id: UniqueId,
        tuplet_id: UniqueId | null,
        tone_id: UniqueId,
        articulation: Articulation,
      ) => {
        return invoke<void>("score_entries_tone_articulation_set", {
          track_id,
          tuplet_id: tuplet_id ?? null,
          tone_id,
          articulation,
        });
      },
    },
    tuplet: {
      create: (
        track_id: UniqueId,
        tick: Tick,
        note_count: number,
        in_space_of: number,
        base_note_value: NoteDuration,
        groupings: number[],
      ) => {
        return invoke<UniqueId>("score_entries_tuplet_create", {
          track_id,
          tick,
          note_count,
          in_space_of,
          base_note_value,
          groupings,
        });
      },
      remove: (track_id: UniqueId, tuplet_id: UniqueId) => {
        return invoke<void>("score_entries_tuplet_remove", {
          track_id,
          tuplet_id,
        });
      },
    },
  },
  file: {
    save: async () => {
      const currentFilePath = await invoke<string | null>(
        "get_current_file_path",
      );

      if (!currentFilePath) {
        // No current file, fall back to save as
        return actions.file.saveAs();
      }

      console.log("Saving project to:", currentFilePath);

      await invoke("save_project_to_file", {
        file_path: currentFilePath,
      });

      // Add to recent files
      await invoke("add_recent_file", {
        file_path: currentFilePath,
      });
    },
    // Save As (always prompt for location)
    saveAs: async () => {
      // Show save dialog
      const filePath = await save({
        title: "Save Project",
        defaultPath: `untitled.soloc`,
        filters: [
          {
            name: "Solo Composer Project",
            extensions: ["soloc"],
          },
        ],
      });

      if (!filePath) {
        throw new Error("Save cancelled by user");
      }

      console.log("Saving project to:", filePath);

      await invoke("save_project_to_file", {
        file_path: filePath,
      });

      // Add to recent files
      await invoke("add_recent_file", {
        file_path: filePath,
      });
    },
    load: async () => {
      // Show open dialog
      const filePath = await open({
        title: "Open Project",
        filters: [
          {
            name: "Solo Composer Project",
            extensions: ["soloc"],
          },
        ],
      });

      if (!filePath || Array.isArray(filePath)) {
        throw new Error("Load cancelled or invalid file selection");
      }

      console.log("Loading project from:", filePath);

      await invoke("load_project_from_file", {
        file_path: filePath,
      });

      // Add to recent files
      await invoke("add_recent_file", {
        file_path: filePath,
      });
    },
    // Load a project from a specific file path (used by recents menu)
    loadFromPath: async (filePath: string) => {
      await invoke("load_project_from_file", {
        file_path: filePath,
      });

      await invoke("add_recent_file", {
        file_path: filePath,
      });

      console.log("Recent file added successfully");
    },
  },
  transport: {
    play: () => invoke("transport_play"),
    pause: () => invoke("transport_pause"),
    seek: (tick: number) => invoke("transport_seek", { tick }),
    getState: () => invoke<TransportState>("transport_get_state"),
    auditionTone: (instrument_id: UniqueId, pitch_int: number) =>
      invoke("playback_audition_tone", { instrument_id, pitch_int }),
  },
};
