/**
 * Comprehensive tests for services/actions.ts
 *
 * Strategy: mock invoke + dialog helpers, call every action, verify the
 * correct Tauri command name and argument object are forwarded.
 */
import { describe, it, expect, vi, beforeEach } from "vitest";

// ---------------------------------------------------------------------------
// Mocks — declared before the module-under-test is imported
// ---------------------------------------------------------------------------
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn().mockResolvedValue("/default/path.soloc"),
  open: vi.fn().mockResolvedValue("/default/open.soloc"),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: Object.assign(
    vi.fn(function MockWebviewWindow() {
      return {
        show: vi.fn(),
        setFocus: vi.fn(),
        once: vi.fn().mockResolvedValue(vi.fn()),
        close: vi.fn(),
      };
    }),
    {
      getByLabel: vi.fn().mockResolvedValue(null),
    },
  ),
}));

import { invoke } from "@tauri-apps/api/core";
import { save, open } from "@tauri-apps/plugin-dialog";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { actions } from "./actions";
import {
  Accidental,
  Articulation,
  AutoCountStyle,
  BarlineDrawType,
  BottomPanelTab,
  BracketingApproach,
  BracketStyle,
  ClefDrawType,
  KeySignatureMode,
  LayoutType,
  NoteDuration,
  PlayingTechniqueKind,
  PlayerType,
  SetupSelectionType,
  Tab,
  TimeSignatureDrawType,
  Tool,
  ExpressionMapEqualityComparator,
} from "./types";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  vi.clearAllMocks();
  mockInvoke.mockResolvedValue(undefined);
});

// ---------------------------------------------------------------------------
// Top-level
// ---------------------------------------------------------------------------
describe("actions.initialize", () => {
  it("calls invoke('initialize')", async () => {
    await actions.initialize();
    expect(mockInvoke).toHaveBeenCalledWith("initialize");
  });
});

describe("actions.reset", () => {
  it("calls invoke('reset')", async () => {
    await actions.reset();
    expect(mockInvoke).toHaveBeenCalledWith("reset");
  });
});

describe("actions.openPluginManager", () => {
  it("creates, shows, and focuses the dedicated plugin manager window", async () => {
    vi.mocked(WebviewWindow.getByLabel).mockResolvedValueOnce(null);

    await actions.openPluginManager();

    const windowMock = vi.mocked(WebviewWindow).mock.results[0]?.value as {
      show: ReturnType<typeof vi.fn>;
      setFocus: ReturnType<typeof vi.fn>;
    };

    expect(WebviewWindow.getByLabel).toHaveBeenCalledWith("plugin-manager");
    expect(WebviewWindow).toHaveBeenCalledWith(
      "plugin-manager",
      expect.objectContaining({
        url: "/plugin-manager.html",
        title: "Plugin Manager",
        visible: false,
      }),
    );
    expect(windowMock).toBeDefined();
    expect(windowMock.show).toHaveBeenCalledTimes(1);
    expect(windowMock.setFocus).toHaveBeenCalledTimes(1);
  });

  it("reuses and focuses the existing dedicated plugin manager window", async () => {
    const existingWindow = {
      show: vi.fn(),
      setFocus: vi.fn(),
    };

    vi.mocked(WebviewWindow.getByLabel).mockResolvedValueOnce(
      existingWindow as unknown as WebviewWindow,
    );

    const result = await actions.openPluginManager();

    expect(WebviewWindow).not.toHaveBeenCalled();
    expect(existingWindow.show).toHaveBeenCalledTimes(1);
    expect(existingWindow.setFocus).toHaveBeenCalledTimes(1);
    expect(result).toBe(existingWindow);
  });
});

describe("actions.openExpressionMapWindow", () => {
  it("creates, shows, and focuses the expression map manager window without a forced selection", async () => {
    vi.mocked(WebviewWindow.getByLabel).mockResolvedValueOnce(null);

    await actions.openExpressionMapManager();

    const windowMock = vi.mocked(WebviewWindow).mock.results[0]?.value as {
      show: ReturnType<typeof vi.fn>;
      setFocus: ReturnType<typeof vi.fn>;
    };

    expect(WebviewWindow.getByLabel).toHaveBeenCalledWith("expression-map");
    expect(WebviewWindow).toHaveBeenCalledWith(
      "expression-map",
      expect.objectContaining({
        url: "/expression-map.html",
        title: "Expression Map",
        visible: false,
      }),
    );
    expect(windowMock.show).toHaveBeenCalledTimes(1);
    expect(windowMock.setFocus).toHaveBeenCalledTimes(1);
  });

  it("reuses the existing expression map manager window and resets it to the menu-owned selection", async () => {
    const existingWindow = {
      show: vi.fn(),
      setFocus: vi.fn(),
      emit: vi.fn().mockResolvedValue(undefined),
    };

    vi.mocked(WebviewWindow.getByLabel).mockResolvedValueOnce(
      existingWindow as unknown as WebviewWindow,
    );

    const result = await actions.openExpressionMapManager();

    expect(WebviewWindow).not.toHaveBeenCalled();
    expect(existingWindow.emit).toHaveBeenCalledWith(
      "soloc://expression-map/select",
      { expression_map_id: "" },
    );
    expect(existingWindow.show).toHaveBeenCalledTimes(1);
    expect(existingWindow.setFocus).toHaveBeenCalledTimes(1);
    expect(result).toBe(existingWindow);
  });

  it("creates, shows, and focuses the dedicated expression map window for a selected map", async () => {
    vi.mocked(WebviewWindow.getByLabel).mockResolvedValueOnce(null);

    await actions.openExpressionMapWindow("legato");

    const windowMock = vi.mocked(WebviewWindow).mock.results[0]?.value as {
      show: ReturnType<typeof vi.fn>;
      setFocus: ReturnType<typeof vi.fn>;
    };

    expect(WebviewWindow.getByLabel).toHaveBeenCalledWith("expression-map");
    expect(WebviewWindow).toHaveBeenCalledWith(
      "expression-map",
      expect.objectContaining({
        url: "/expression-map.html?expression_map_id=legato",
        title: "Expression Map",
        visible: false,
      }),
    );
    expect(windowMock.show).toHaveBeenCalledTimes(1);
    expect(windowMock.setFocus).toHaveBeenCalledTimes(1);
  });

  it("reuses the existing expression map window and retargets it to the selected map", async () => {
    const existingWindow = {
      show: vi.fn(),
      setFocus: vi.fn(),
      emit: vi.fn().mockResolvedValue(undefined),
    };

    vi.mocked(WebviewWindow.getByLabel).mockResolvedValueOnce(
      existingWindow as unknown as WebviewWindow,
    );

    const result = await actions.openExpressionMapWindow("staccato");

    expect(WebviewWindow).not.toHaveBeenCalled();
    expect(existingWindow.emit).toHaveBeenCalledWith(
      "soloc://expression-map/select",
      { expression_map_id: "staccato" },
    );
    expect(existingWindow.show).toHaveBeenCalledTimes(1);
    expect(existingWindow.setFocus).toHaveBeenCalledTimes(1);
    expect(result).toBe(existingWindow);
  });
});

// ---------------------------------------------------------------------------
// audio
// ---------------------------------------------------------------------------
describe("actions.audio", () => {
  it("setDevice calls invoke with device_id", async () => {
    await actions.audio.setDevice("hw:0");
    expect(mockInvoke).toHaveBeenCalledWith("audio_device_set", {
      device_id: "hw:0",
    });
  });

  it("setDeviceSampleRate calls invoke with sample_rate", async () => {
    await actions.audio.setDeviceSampleRate(48000);
    expect(mockInvoke).toHaveBeenCalledWith("audio_device_sample_rate_set", {
      sample_rate: 48000,
    });
  });

  it("refreshDevices calls invoke correctly", async () => {
    await actions.audio.refreshDevices();
    expect(mockInvoke).toHaveBeenCalledWith("audio_devices_refresh");
  });
});

// ---------------------------------------------------------------------------
// pluginInventory
// ---------------------------------------------------------------------------
describe("actions.pluginInventory", () => {
  it("discover calls invoke correctly", async () => {
    await actions.pluginInventory.discover();
    expect(mockInvoke).toHaveBeenCalledWith("plugin_inventory_discover");
  });

  it("openEditorHost calls invoke with inventory-backed plugin_id", async () => {
    await actions.pluginInventory.openEditorHost(
      "vst3:/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
    );
    expect(mockInvoke).toHaveBeenCalledWith("plugin_host_open", {
      slot_id: null,
      plugin_id: "vst3:/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
      plugin_path: null,
    });
  });

  it("openEditorHost forwards an optional slot_id for rack-targeted editor opens", async () => {
    await actions.pluginInventory.openEditorHost("vst3:grand-piano", {
      slot_id: 12,
    });
    expect(mockInvoke).toHaveBeenCalledWith("plugin_host_open", {
      slot_id: 12,
      plugin_id: "vst3:grand-piano",
      plugin_path: null,
    });
  });

  it("openEditorHost times out when the backend call never settles", async () => {
    vi.useFakeTimers();
    mockInvoke.mockImplementationOnce(() => new Promise(() => {}));

    try {
      const result = actions.pluginInventory.openEditorHost("vst3:grand-piano");
      await vi.advanceTimersByTimeAsync(12000);

      await expect(result).rejects.toThrow(
        "Opening the plugin UI timed out. Compose may have stalled while initializing the plugin host.",
      );
    } finally {
      vi.useRealTimers();
    }
  });
});

describe("actions.expressionMaps", () => {
  it("getState calls invoke correctly", async () => {
    await actions.expressionMaps.getState();
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_get_state");
  });

  it("create calls invoke with an optional name payload", async () => {
    await actions.expressionMaps.create("Legato");

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_create", {
      name: "Legato",
    });
  });

  it("duplicate calls invoke with expression_map_id payload", async () => {
    await actions.expressionMaps.duplicate("legato");

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_duplicate", {
      expression_map_id: "legato",
    });
  });

  it("metadata setters call invoke with expression_map_id payloads", async () => {
    await actions.expressionMaps.setName("legato", "Legato Deluxe");
    await actions.expressionMaps.setCreator("legato", "Aled");
    await actions.expressionMaps.setVersion("legato", 4);

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_name_set", {
      expression_map_id: "legato",
      name: "Legato Deluxe",
    });
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_creator_set", {
      expression_map_id: "legato",
      creator: "Aled",
    });
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_version_set", {
      expression_map_id: "legato",
      version: 4,
    });
  });

  it("addRule calls invoke with expression_map_id payload", async () => {
    await actions.expressionMaps.addRule("legato");

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_rule_add", {
      expression_map_id: "legato",
    });
  });

  it("setRuleName calls invoke with expression_map_id, rule_id and name", async () => {
    await actions.expressionMaps.setRuleName("legato", "rule-1", "Lead Accent");

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_rule_name_set", {
      expression_map_id: "legato",
      rule_id: "rule-1",
      name: "Lead Accent",
    });
  });

  it("moveRule calls invoke with expression_map_id, rule_id and target index", async () => {
    await actions.expressionMaps.moveRule("legato", "rule-1", 2);

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_rule_move", {
      expression_map_id: "legato",
      rule_id: "rule-1",
      to_index: 2,
    });
  });

  it("removeRule calls invoke with expression_map_id and rule_id", async () => {
    await actions.expressionMaps.removeRule("legato", "rule-1");

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_rule_remove", {
      expression_map_id: "legato",
      rule_id: "rule-1",
    });
  });

  it("nested condition and action commands use id-based payloads", async () => {
    await actions.expressionMaps.addCondition("legato", "rule-1", "no_op");
    await actions.expressionMaps.setConditionType(
      "legato",
      "rule-1",
      "condition-1",
      "written_note_length",
    );
    await actions.expressionMaps.updateCondition("legato", "rule-1", {
      id: "condition-1",
      type: "articulation",
      comparator: ExpressionMapEqualityComparator.Equals,
      articulation: Articulation.Accent,
    });
    await actions.expressionMaps.removeCondition(
      "legato",
      "rule-1",
      "condition-1",
    );
    await actions.expressionMaps.addAction("legato", "rule-1", "no_op");
    await actions.expressionMaps.setActionType(
      "legato",
      "rule-1",
      "action-1",
      "keyswitch_note",
    );
    await actions.expressionMaps.updateAction("legato", "rule-1", {
      id: "action-1",
      type: "keyswitch_note",
      midi_note: 24,
      velocity: 96,
    });
    await actions.expressionMaps.removeAction("legato", "rule-1", "action-1");

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_condition_add", {
      expression_map_id: "legato",
      rule_id: "rule-1",
      condition_type: "no_op",
    });
    expect(mockInvoke).toHaveBeenCalledWith(
      "expression_maps_condition_set_type",
      {
        expression_map_id: "legato",
        rule_id: "rule-1",
        condition_id: "condition-1",
        condition_type: "written_note_length",
      },
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "expression_maps_condition_update",
      {
        expression_map_id: "legato",
        rule_id: "rule-1",
        condition: {
          id: "condition-1",
          type: "articulation",
          comparator: ExpressionMapEqualityComparator.Equals,
          articulation: Articulation.Accent,
        },
      },
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "expression_maps_condition_remove",
      {
        expression_map_id: "legato",
        rule_id: "rule-1",
        condition_id: "condition-1",
      },
    );
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_action_add", {
      expression_map_id: "legato",
      rule_id: "rule-1",
      action_type: "no_op",
    });
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_action_set_type", {
      expression_map_id: "legato",
      rule_id: "rule-1",
      action_id: "action-1",
      action_type: "keyswitch_note",
    });
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_action_update", {
      expression_map_id: "legato",
      rule_id: "rule-1",
      action: {
        id: "action-1",
        type: "keyswitch_note",
        midi_note: 24,
        velocity: 96,
      },
    });
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_action_remove", {
      expression_map_id: "legato",
      rule_id: "rule-1",
      action_id: "action-1",
    });
  });

  it("default action commands use map-level payloads", async () => {
    await actions.expressionMaps.addDefaultAction("legato", "no_op");
    await actions.expressionMaps.setDefaultActionType(
      "legato",
      "default-action-1",
      "keyswitch_note",
    );
    await actions.expressionMaps.updateDefaultAction("legato", {
      id: "default-action-1",
      type: "keyswitch_note",
      midi_note: 24,
      velocity: 96,
    });
    await actions.expressionMaps.removeDefaultAction(
      "legato",
      "default-action-1",
    );

    expect(mockInvoke).toHaveBeenCalledWith(
      "expression_maps_default_action_add",
      {
        expression_map_id: "legato",
        action_type: "no_op",
      },
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "expression_maps_default_action_set_type",
      {
        expression_map_id: "legato",
        action_id: "default-action-1",
        action_type: "keyswitch_note",
      },
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "expression_maps_default_action_update",
      {
        expression_map_id: "legato",
        action: {
          id: "default-action-1",
          type: "keyswitch_note",
          midi_note: 24,
          velocity: 96,
        },
      },
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "expression_maps_default_action_remove",
      {
        expression_map_id: "legato",
        action_id: "default-action-1",
      },
    );
  });

  it("delete calls invoke with expression_map_id payload", async () => {
    await actions.expressionMaps.delete("legato");

    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_delete", {
      expression_map_id: "legato",
    });
  });

  it("importFromDialog opens a JSON picker and imports the selected file", async () => {
    vi.mocked(open).mockResolvedValueOnce("/tmp/expression-maps.json");

    await actions.expressionMaps.importFromDialog();

    expect(open).toHaveBeenCalledWith(
      expect.objectContaining({
        multiple: false,
        title: "Import Expression Maps",
      }),
    );
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_import", {
      file_path: "/tmp/expression-maps.json",
    });
  });

  it("exportToDialog saves the selected expression maps as JSON", async () => {
    vi.mocked(save).mockResolvedValueOnce("/tmp/expression-maps.json");

    await actions.expressionMaps.exportToDialog(["legato", "staccato"]);

    expect(save).toHaveBeenCalledWith(
      expect.objectContaining({
        title: "Export Expression Maps",
      }),
    );
    expect(mockInvoke).toHaveBeenCalledWith("expression_maps_export", {
      file_path: "/tmp/expression-maps.json",
      expression_map_ids: ["legato", "staccato"],
    });
  });
});

describe("actions.score.instrument.setExpressionMap", () => {
  it("calls invoke with instrument_id and expression_map_id payload", async () => {
    await actions.score.instrument.setExpressionMap(12, "legato");

    expect(mockInvoke).toHaveBeenCalledWith(
      "score_instruments_set_expression_map",
      {
        instrument_id: 12,
        expression_map_id: "legato",
      },
    );
  });

  it("supports clearing the current expression map linkage with null", async () => {
    await actions.score.instrument.setExpressionMap(12, null);

    expect(mockInvoke).toHaveBeenCalledWith(
      "score_instruments_set_expression_map",
      {
        instrument_id: 12,
        expression_map_id: null,
      },
    );
  });
});

// ---------------------------------------------------------------------------
// ui.*
// ---------------------------------------------------------------------------
describe("actions.ui", () => {
  it("setTab calls invoke with tab", async () => {
    await actions.ui.setTab(Tab.Write);
    expect(mockInvoke).toHaveBeenCalledWith("ui_tab_set", { tab: Tab.Write });
  });

  it("setActiveLayoutId calls invoke correctly", async () => {
    await actions.ui.setActiveLayoutId(3);
    expect(mockInvoke).toHaveBeenCalledWith("ui_layout_id_set", {
      layout_id: 3,
    });
  });

  it("setActiveFlowId calls invoke correctly", async () => {
    await actions.ui.setActiveFlowId(7);
    expect(mockInvoke).toHaveBeenCalledWith("ui_flow_id_set", { flow_id: 7 });
  });

  describe("preferences", () => {
    it("setAudition calls invoke correctly", async () => {
      await actions.ui.preferences.setAudition(true);
      expect(mockInvoke).toHaveBeenCalledWith("preferences_audition_set", {
        audition: true,
      });
    });
  });

  describe("renderer", () => {
    it("setZoom calls invoke with zoom", async () => {
      await actions.ui.renderer.setZoom(1.5);
      expect(mockInvoke).toHaveBeenCalledWith("ui_renderer_zoom_set", {
        zoom: 1.5,
      });
    });

    it("setPan calls invoke with x,y", async () => {
      await actions.ui.renderer.setPan(10, 20);
      expect(mockInvoke).toHaveBeenCalledWith("ui_renderer_pan_set", {
        x: 10,
        y: 20,
      });
    });

    it("setBounds calls invoke with bounds and px_per_mm", async () => {
      const bounds = { x: 0, y: 0, width: 800, height: 600 };
      await actions.ui.renderer.setBounds(3.78, bounds);
      expect(mockInvoke).toHaveBeenCalledWith("ui_renderer_bounds_set", {
        bounds,
        px_per_mm: 3.78,
      });
    });

    it("toggleDebug calls invoke correctly", async () => {
      await actions.ui.renderer.toggleDebug();
      expect(mockInvoke).toHaveBeenCalledWith("ui_renderer_debug_toggle");
    });

    it("getSelectionCoordinates calls invoke correctly", async () => {
      mockInvoke.mockResolvedValueOnce([5, 10]);
      const result = await actions.ui.renderer.getSelectionCoordinates();
      expect(mockInvoke).toHaveBeenCalledWith("ui_get_selection_coordinates");
      expect(result).toEqual([5, 10]);
    });
  });

  describe("sequencer", () => {
    it("setInstrument calls invoke with id", async () => {
      await actions.ui.sequencer.setInstrument(42);
      expect(mockInvoke).toHaveBeenCalledWith("ui_sequencer_instrument_set", {
        id: 42,
      });
    });

    it("setTrack calls invoke with id", async () => {
      await actions.ui.sequencer.setTrack(5);
      expect(mockInvoke).toHaveBeenCalledWith("ui_sequencer_track_set", {
        id: 5,
      });
    });

    it("setTool calls invoke with tool", async () => {
      await actions.ui.sequencer.setTool(Tool.Write);
      expect(mockInvoke).toHaveBeenCalledWith("ui_sequencer_tool_set", {
        tool: Tool.Write,
      });
    });

    it("setSnap calls invoke with snap", async () => {
      await actions.ui.sequencer.setSnap(NoteDuration.Eighth);
      expect(mockInvoke).toHaveBeenCalledWith("ui_sequencer_snap_set", {
        snap: NoteDuration.Eighth,
      });
    });

    it("setCoords calls invoke with x,y", async () => {
      await actions.ui.sequencer.setCoords(100, 200);
      expect(mockInvoke).toHaveBeenCalledWith("ui_sequencer_coords_set", {
        x: 100,
        y: 200,
      });
    });

    it("selectTone calls invoke with all IDs", async () => {
      await actions.ui.sequencer.selectTone(1, 2, 3, 4, 5);
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequencer_selection_set_tone",
        {
          tone_id: 1,
          track_id: 2,
          stave_id: 3,
          flow_id: 4,
          instrument_id: 5,
          tuplet_id: null,
        },
      );
    });

    it("selectRest calls invoke with all IDs", async () => {
      await actions.ui.sequencer.selectRest(64, 2, 3, 4, 5);
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequencer_selection_set_rest",
        {
          tick: 64,
          track_id: 2,
          stave_id: 3,
          flow_id: 4,
          instrument_id: 5,
          tuplet_id: null,
        },
      );
    });

    it("selectTempo calls invoke with the tempo selection payload", async () => {
      await actions.ui.sequencer.selectTempo(8, 4, 64);
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequencer_selection_set_tempo",
        {
          tempo_id: 8,
          flow_id: 4,
          tick: 64,
        },
      );
    });

    it("selectTimeSignature calls invoke with the time signature selection payload", async () => {
      await actions.ui.sequencer.selectTimeSignature(4, 8, 64);
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequencer_selection_set_time_signature",
        {
          flow_id: 4,
          time_signature_id: 8,
          tick: 64,
        },
      );
    });

    it("selectKeySignature calls invoke with the key signature selection payload", async () => {
      await actions.ui.sequencer.selectKeySignature(4, 9, 64);
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequencer_selection_set_key_signature",
        {
          flow_id: 4,
          key_signature_id: 9,
          tick: 64,
        },
      );
    });

    it("selectClef calls invoke with the clef selection payload", async () => {
      await actions.ui.sequencer.selectClef(4, 3, 10, 64);
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequencer_selection_set_clef",
        {
          flow_id: 4,
          stave_id: 3,
          clef_id: 10,
          tick: 64,
        },
      );
    });

    it("selectPlayingTechnique calls invoke with the playing-technique selection payload", async () => {
      await actions.ui.sequencer.selectPlayingTechnique(4, 3, 10, 64);
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequencer_selection_set_playing_technique",
        {
          flow_id: 4,
          stave_id: 3,
          playing_technique_id: 10,
          tick: 64,
        },
      );
    });

    it("selectTuplet calls invoke with the tuplet selection payload", async () => {
      await actions.ui.sequencer.selectTuplet(4, 5, 3, 2, 10, 64);
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequencer_selection_set_tuplet",
        {
          flow_id: 4,
          instrument_id: 5,
          stave_id: 3,
          track_id: 2,
          tuplet_id: 10,
          tick: 64,
        },
      );
    });

    it("clearSelection calls invoke correctly", async () => {
      await actions.ui.sequencer.clearSelection();
      expect(mockInvoke).toHaveBeenCalledWith("ui_sequencer_selection_clear");
    });
  });

  describe("screens.setup", () => {
    it("select calls invoke with selection_type and id", async () => {
      await actions.ui.screens.setup.select(SetupSelectionType.Player, 1);
      expect(mockInvoke).toHaveBeenCalledWith("ui_setup_selection_set", {
        selection_type: SetupSelectionType.Player,
        id: 1,
      });
    });

    it("clearSelection calls invoke correctly", async () => {
      await actions.ui.screens.setup.clearSelection();
      expect(mockInvoke).toHaveBeenCalledWith("ui_setup_selection_clear");
    });

    it("expand calls invoke with id", async () => {
      await actions.ui.screens.setup.expand(10);
      expect(mockInvoke).toHaveBeenCalledWith("ui_setup_expanded_insert", {
        id: 10,
      });
    });

    it("collapse calls invoke with id", async () => {
      await actions.ui.screens.setup.collapse(10);
      expect(mockInvoke).toHaveBeenCalledWith("ui_setup_expanded_remove", {
        id: 10,
      });
    });
  });

  describe("screens.write.bottomPanel", () => {
    it("toggle calls invoke correctly", async () => {
      await actions.ui.screens.write.bottomPanel.toggle();
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_write_bottom_panel_minimized_toggle",
      );
    });

    it("setTab calls invoke with tab", async () => {
      await actions.ui.screens.write.bottomPanel.setTab(
        BottomPanelTab.Sequencer,
      );
      expect(mockInvoke).toHaveBeenCalledWith("ui_write_bottom_panel_tab_set", {
        tab: BottomPanelTab.Sequencer,
      });
    });
  });

  describe("screens.sequence.bottomPanel", () => {
    it("toggle calls invoke correctly", async () => {
      await actions.ui.screens.sequence.bottomPanel.toggle();
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequence_bottom_panel_minimized_toggle",
      );
    });

    it("setTab calls invoke with tab", async () => {
      await actions.ui.screens.sequence.bottomPanel.setTab(
        BottomPanelTab.Mixer,
      );
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequence_bottom_panel_tab_set",
        { tab: BottomPanelTab.Mixer },
      );
    });

    it("instruments.setShowActiveOnly calls invoke correctly", async () => {
      await actions.ui.screens.sequence.bottomPanel.instruments.setShowActiveOnly(
        true,
      );
      expect(mockInvoke).toHaveBeenCalledWith(
        "ui_sequence_show_active_only_set",
        { show_active_only: true },
      );
    });
  });
});

// ---------------------------------------------------------------------------
// score.engrave
// ---------------------------------------------------------------------------
describe("actions.score.engrave", () => {
  const lt = LayoutType.FullScore;

  it("setSystemicBarlineSingleInstrumentSystem", async () => {
    await actions.score.engrave.setSystemicBarlineSingleInstrumentSystem(
      lt,
      true,
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_systemic_barline_single_instrument_system_set",
      { layout_type: lt, value: true },
    );
  });

  it("setBracketingApproach", async () => {
    await actions.score.engrave.setBracketingApproach(
      lt,
      BracketingApproach.Orchestral,
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_bracketing_approach_set",
      { layout_type: lt, value: BracketingApproach.Orchestral },
    );
  });

  it("setBracketStyle", async () => {
    await actions.score.engrave.setBracketStyle(lt, BracketStyle.Line);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_bracket_style_set",
      { layout_type: lt, value: BracketStyle.Line },
    );
  });

  it("setBracketSingleStaves", async () => {
    await actions.score.engrave.setBracketSingleStaves(lt, false);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_bracket_single_staves_set",
      { layout_type: lt, value: false },
    );
  });

  it("setBaseNoteSpace", async () => {
    await actions.score.engrave.setBaseNoteSpace(lt, 12);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_base_note_space_set",
      { layout_type: lt, value: 12 },
    );
  });

  it("setMinimumNoteSpace", async () => {
    await actions.score.engrave.setMinimumNoteSpace(lt, 5);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_minimum_note_space_set",
      { layout_type: lt, value: 5 },
    );
  });

  it("setMinimumTieSpace", async () => {
    await actions.score.engrave.setMinimumTieSpace(lt, 3);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_minimum_tie_space_set",
      { layout_type: lt, value: 3 },
    );
  });

  it("setNoteSpaceRatio", async () => {
    await actions.score.engrave.setNoteSpaceRatio(lt, 1.4);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_note_space_ratio_set",
      { layout_type: lt, value: 1.4 },
    );
  });

  it("setSpace", async () => {
    await actions.score.engrave.setSpace(lt, 8);
    expect(mockInvoke).toHaveBeenCalledWith("score_engraves_space_set", {
      layout_type: lt,
      value: 8,
    });
  });

  it("setFramePaddingTop", async () => {
    await actions.score.engrave.setFramePaddingTop(lt, 10);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_frame_padding_top_set",
      { layout_type: lt, value: 10 },
    );
  });

  it("setFramePaddingBottom", async () => {
    await actions.score.engrave.setFramePaddingBottom(lt, 10);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_frame_padding_bottom_set",
      { layout_type: lt, value: 10 },
    );
  });

  it("setPagePaddingTop", async () => {
    await actions.score.engrave.setPagePaddingTop(lt, 20);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_page_padding_top_set",
      { layout_type: lt, value: 20 },
    );
  });

  it("setPagePaddingRight", async () => {
    await actions.score.engrave.setPagePaddingRight(lt, 15);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_page_padding_right_set",
      { layout_type: lt, value: 15 },
    );
  });

  it("setPagePaddingBottom", async () => {
    await actions.score.engrave.setPagePaddingBottom(lt, 20);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_page_padding_bottom_set",
      { layout_type: lt, value: 20 },
    );
  });

  it("setPagePaddingLeft", async () => {
    await actions.score.engrave.setPagePaddingLeft(lt, 15);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_engraves_page_padding_left_set",
      { layout_type: lt, value: 15 },
    );
  });
});

describe("actions.score.tempo", () => {
  it("create calls invoke with the tempo payload", async () => {
    await actions.score.tempo.create(
      5,
      64,
      "Allegro",
      NoteDuration.Quarter,
      0,
      120,
      false,
      true,
      true,
      false,
    );

    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tempo_create", {
      flow_id: 5,
      tick: 64,
      text: "Allegro",
      beat_type: NoteDuration.Quarter,
      dots: 0,
      bpm: 120,
      hidden: false,
      show_text: true,
      show_metronome: true,
      parenthesized: false,
    });
  });

  it("update calls invoke with the full tempo update payload", async () => {
    await actions.score.tempo.update(
      5,
      9,
      "Andante",
      NoteDuration.Eighth,
      1,
      96,
      false,
      true,
      true,
      true,
    );

    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tempo_update", {
      flow_id: 5,
      tempo_id: 9,
      text: "Andante",
      beat_type: NoteDuration.Eighth,
      dots: 1,
      bpm: 96,
      hidden: false,
      show_text: true,
      show_metronome: true,
      parenthesized: true,
    });
  });

  it("clearAtTick calls invoke correctly", async () => {
    await actions.score.tempo.clearAtTick(5, 64);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tempo_clear", {
      flow_id: 5,
      tick: 64,
    });
  });
});

describe("actions.score.playingTechnique", () => {
  it("create calls invoke with the playing-technique payload", async () => {
    await actions.score.playingTechnique.create(
      5,
      64,
      7,
      PlayingTechniqueKind.Pizz,
      true,
    );

    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_playing_technique_create",
      {
        flow_id: 5,
        tick: 64,
        stave_id: 7,
        kind: PlayingTechniqueKind.Pizz,
        hidden: true,
      },
    );
  });

  it("remove calls invoke with the playing-technique identity payload", async () => {
    await actions.score.playingTechnique.remove(5, 7, 9);

    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_playing_technique_remove",
      {
        flow_id: 5,
        stave_id: 7,
        playing_technique_id: 9,
      },
    );
  });

  it("update calls invoke with the bounded hidden payload", async () => {
    await actions.score.playingTechnique.update(5, 7, 9, true);

    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_playing_technique_update",
      {
        flow_id: 5,
        stave_id: 7,
        playing_technique_id: 9,
        hidden: true,
      },
    );
  });
});

// ---------------------------------------------------------------------------
// score.flow
// ---------------------------------------------------------------------------
describe("actions.score.flow", () => {
  it("create returns the new flow id", async () => {
    mockInvoke.mockResolvedValueOnce(5);
    const id = await actions.score.flow.create();
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_create");
    expect(id).toBe(5);
  });

  it("rename calls invoke correctly", async () => {
    await actions.score.flow.rename(1, "My Flow");
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_rename", {
      flow_id: 1,
      title: "My Flow",
    });
  });

  it("move calls invoke with indices", async () => {
    await actions.score.flow.move(0, 2);
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_move", {
      from_index: 0,
      to_index: 2,
    });
  });

  it("insertTime accepts an arbitrary tick duration", async () => {
    await actions.score.flow.insertTime(1, 65, 17);
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_insert_time", {
      flow_id: 1,
      tick: 65,
      duration: 17,
    });
  });

  it("deleteTime accepts an arbitrary tick range", async () => {
    await actions.score.flow.deleteTime(1, 65, 17);
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_delete_time", {
      flow_id: 1,
      start_tick: 65,
      duration: 17,
    });
  });

  it("insertBarAfter returns the inserted bar tick", async () => {
    mockInvoke.mockResolvedValueOnce(256);
    const tick = await actions.score.flow.insertBarAfter(1, 128);
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_insert_bar_after", {
      flow_id: 1,
      bar_start_tick: 128,
    });
    expect(tick).toBe(256);
  });

  it("deleteBar returns the adjacent bar tick", async () => {
    mockInvoke.mockResolvedValueOnce(128);
    const tick = await actions.score.flow.deleteBar(1, 256);
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_delete_bar", {
      flow_id: 1,
      bar_start_tick: 256,
    });
    expect(tick).toBe(128);
  });

  it("assignPlayer calls invoke correctly", async () => {
    await actions.score.flow.assignPlayer(1, 10);
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_assign_player", {
      flow_id: 1,
      player_id: 10,
    });
  });

  it("unassignPlayer calls invoke correctly", async () => {
    await actions.score.flow.unassignPlayer(1, 10);
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_unassign_player", {
      flow_id: 1,
      player_id: 10,
    });
  });

  it("remove calls invoke with flow_id", async () => {
    await actions.score.flow.remove(3);
    expect(mockInvoke).toHaveBeenCalledWith("score_flows_remove", {
      flow_id: 3,
    });
  });
});

// ---------------------------------------------------------------------------
// score.layout
// ---------------------------------------------------------------------------
describe("actions.score.layout", () => {
  it("create returns the new layout id", async () => {
    mockInvoke.mockResolvedValueOnce(9);
    const id = await actions.score.layout.create(LayoutType.Part);
    expect(mockInvoke).toHaveBeenCalledWith("score_layouts_create", {
      layout_type: LayoutType.Part,
    });
    expect(id).toBe(9);
  });

  it("rename calls invoke correctly", async () => {
    await actions.score.layout.rename(2, "Part A");
    expect(mockInvoke).toHaveBeenCalledWith("score_layouts_rename", {
      layout_id: 2,
      title: "Part A",
    });
  });

  it("assignPlayer calls invoke correctly", async () => {
    await actions.score.layout.assignPlayer(2, 5);
    expect(mockInvoke).toHaveBeenCalledWith("score_layouts_assign_player", {
      layout_id: 2,
      player_id: 5,
    });
  });

  it("unassignPlayer calls invoke correctly", async () => {
    await actions.score.layout.unassignPlayer(2, 5);
    expect(mockInvoke).toHaveBeenCalledWith("score_layouts_unassign_player", {
      layout_id: 2,
      player_id: 5,
    });
  });

  it("assignFlow calls invoke correctly", async () => {
    await actions.score.layout.assignFlow(2, 3);
    expect(mockInvoke).toHaveBeenCalledWith("score_layouts_assign_flow", {
      layout_id: 2,
      flow_id: 3,
    });
  });

  it("unassignFlow calls invoke correctly", async () => {
    await actions.score.layout.unassignFlow(2, 3);
    expect(mockInvoke).toHaveBeenCalledWith("score_layouts_unassign_flow", {
      layout_id: 2,
      flow_id: 3,
    });
  });

  it("move calls invoke correctly", async () => {
    await actions.score.layout.move(0, 1);
    expect(mockInvoke).toHaveBeenCalledWith("score_layouts_move", {
      from_index: 0,
      to_index: 1,
    });
  });

  it("remove calls invoke correctly", async () => {
    await actions.score.layout.remove(4);
    expect(mockInvoke).toHaveBeenCalledWith("score_layouts_remove", {
      layout_id: 4,
    });
  });
});

// ---------------------------------------------------------------------------
// score.player
// ---------------------------------------------------------------------------
describe("actions.score.player", () => {
  it("create returns the new player id", async () => {
    mockInvoke.mockResolvedValueOnce(7);
    const id = await actions.score.player.create(PlayerType.Solo);
    expect(mockInvoke).toHaveBeenCalledWith("score_players_create", {
      player_type: PlayerType.Solo,
    });
    expect(id).toBe(7);
  });

  it("assignInstrument calls invoke correctly", async () => {
    await actions.score.player.assignInstrument(1, 20);
    expect(mockInvoke).toHaveBeenCalledWith("score_players_assign_instrument", {
      player_id: 1,
      instrument_id: 20,
    });
  });

  it("unassignInstrument calls invoke correctly", async () => {
    await actions.score.player.unassignInstrument(1, 20);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_players_unassign_instrument",
      { player_id: 1, instrument_id: 20 },
    );
  });

  it("move calls invoke correctly", async () => {
    await actions.score.player.move(0, 2);
    expect(mockInvoke).toHaveBeenCalledWith("score_players_move", {
      from_index: 0,
      to_index: 2,
    });
  });

  it("remove calls invoke correctly", async () => {
    await actions.score.player.remove(3);
    expect(mockInvoke).toHaveBeenCalledWith("score_players_remove", {
      player_id: 3,
    });
  });
});

// ---------------------------------------------------------------------------
// score.instrument
// ---------------------------------------------------------------------------
describe("actions.score.instrument", () => {
  it("create returns the new instrument id", async () => {
    mockInvoke.mockResolvedValueOnce(11);
    const id = await actions.score.instrument.create("piano");
    expect(mockInvoke).toHaveBeenCalledWith("score_instruments_create", {
      definition_id: "piano",
    });
    expect(id).toBe(11);
  });

  it("move calls invoke correctly", async () => {
    await actions.score.instrument.move(0, 1, 2, 3);
    expect(mockInvoke).toHaveBeenCalledWith("score_instruments_move", {
      from_index: 0,
      to_index: 1,
      from_player_id: 2,
      to_player_id: 3,
    });
  });

  it("remove calls invoke correctly", async () => {
    await actions.score.instrument.remove(10);
    expect(mockInvoke).toHaveBeenCalledWith("score_instruments_remove", {
      instrument_id: 10,
    });
  });

  it("assignSlotRoute calls invoke with explicit target and address", async () => {
    await actions.score.instrument.assignSlotRoute(
      10,
      30,
      { mode: 0, index: null },
      { mode: 0, index: null },
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_instruments_assign_slot_route",
      {
        instrument_id: 10,
        route: {
          slot_id: 30,
          target: { mode: 0, index: null },
          address: { mode: 0, index: null },
        },
      },
    );
  });

  it("unassignRoute calls invoke correctly", async () => {
    await actions.score.instrument.unassignRoute(10);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_instruments_unassign_route",
      {
        instrument_id: 10,
      },
    );
  });

  it("create playback slot calls invoke correctly", async () => {
    await actions.score.playback.slot.create();
    expect(mockInvoke).toHaveBeenCalledWith("score_playback_slots_create");
  });

  it("remove playback slot calls invoke correctly", async () => {
    await actions.score.playback.slot.remove(55);
    expect(mockInvoke).toHaveBeenCalledWith("score_playback_slots_remove", {
      slot_id: 55,
    });
  });

  it("assignPlugin to playback slot calls invoke correctly", async () => {
    await actions.score.playback.slot.assignPlugin(55, "vst-1");
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_playback_slots_assign_plugin",
      {
        slot_id: 55,
        plugin_id: "vst-1",
      },
    );
  });

  it("assignPlugin to playback slot times out when the backend call never settles", async () => {
    vi.useFakeTimers();
    mockInvoke.mockImplementationOnce(() => new Promise(() => {}));

    try {
      const result = actions.score.playback.slot.assignPlugin(55, "vst-1");
      await vi.advanceTimersByTimeAsync(12000);

      await expect(result).rejects.toThrow(
        "Assigning the plugin to the slot timed out. Compose may have stalled while initializing the plugin.",
      );
    } finally {
      vi.useRealTimers();
    }
  });

  it("unassignPlugin from playback slot calls invoke correctly", async () => {
    await actions.score.playback.slot.unassignPlugin(55);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_playback_slots_unassign_plugin",
      {
        slot_id: 55,
      },
    );
  });

  it("setVolume calls invoke correctly", async () => {
    await actions.score.instrument.setVolume(10, 80);
    expect(mockInvoke).toHaveBeenCalledWith("score_instruments_set_volume", {
      instrument_id: 10,
      volume: 80,
    });
  });

  it("setPan calls invoke correctly", async () => {
    await actions.score.instrument.setPan(10, -50);
    expect(mockInvoke).toHaveBeenCalledWith("score_instruments_set_pan", {
      instrument_id: 10,
      pan: -50,
    });
  });

  it("setMute calls invoke correctly", async () => {
    await actions.score.instrument.setMute(10, true);
    expect(mockInvoke).toHaveBeenCalledWith("score_instruments_set_mute", {
      instrument_id: 10,
      mute: true,
    });
  });

  it("setSolo calls invoke correctly", async () => {
    await actions.score.instrument.setSolo(10, false);
    expect(mockInvoke).toHaveBeenCalledWith("score_instruments_set_solo", {
      instrument_id: 10,
      solo: false,
    });
  });
});

// ---------------------------------------------------------------------------
// score.config
// ---------------------------------------------------------------------------
describe("actions.score.config", () => {
  it("autoCountStyles.setSolo calls invoke correctly", async () => {
    await actions.score.config.autoCountStyles.setSolo(AutoCountStyle.Arabic);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_config_auto_count_styles_solo_set",
      { auto_count_style: AutoCountStyle.Arabic },
    );
  });

  it("autoCountStyles.setSection calls invoke correctly", async () => {
    await actions.score.config.autoCountStyles.setSection(AutoCountStyle.Roman);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_config_auto_count_styles_section_set",
      { auto_count_style: AutoCountStyle.Roman },
    );
  });
});

// ---------------------------------------------------------------------------
// score.meta
// ---------------------------------------------------------------------------
describe("actions.score.meta", () => {
  const metaCases: Array<[keyof typeof actions.score.meta, string, string]> = [
    ["setTitle", "score_meta_title_set", "title"],
    ["setSubtitle", "score_meta_subtitle_set", "subtitle"],
    ["setDedication", "score_meta_dedication_set", "dedication"],
    ["setComposer", "score_meta_composer_set", "composer"],
    ["setArranger", "score_meta_arranger_set", "arranger"],
    ["setLyricist", "score_meta_lyricist_set", "lyricist"],
    ["setArtist", "score_meta_artist_set", "artist"],
    ["setCopyist", "score_meta_copyist_set", "copyist"],
    ["setPublisher", "score_meta_publisher_set", "publisher"],
    ["setEditor", "score_meta_editor_set", "editor"],
    ["setCopyright", "score_meta_copyright_set", "copyright"],
    ["setWorkNumber", "score_meta_work_number_set", "work_number"],
    [
      "setCompositionYear",
      "score_meta_composition_year_set",
      "composition_year",
    ],
    ["setComposerDates", "score_meta_composer_dates_set", "composer_dates"],
    [
      "setOtherInformation",
      "score_meta_other_information_set",
      "other_information",
    ],
  ];

  it.each(metaCases)(
    "%s calls invoke(%s, { %s: value })",
    async (method, command, argKey) => {
      await (actions.score.meta[method] as (v: string) => Promise<void>)(
        "test-value",
      );
      expect(mockInvoke).toHaveBeenCalledWith(command, {
        [argKey]: "test-value",
      });
    },
  );
});

// ---------------------------------------------------------------------------
// score.barline
// ---------------------------------------------------------------------------
describe("actions.score.barline", () => {
  it("create calls invoke correctly", async () => {
    mockInvoke.mockResolvedValueOnce(99);
    const id = await actions.score.barline.create(1, 64, BarlineDrawType.Final);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_barline_create", {
      flow_id: 1,
      tick: 64,
      barline_draw_type: BarlineDrawType.Final,
    });
    expect(id).toBe(99);
  });

  it("clearAtTick calls invoke correctly", async () => {
    await actions.score.barline.clearAtTick(1, 64);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_barline_clear", {
      flow_id: 1,
      tick: 64,
    });
  });
});

// ---------------------------------------------------------------------------
// score.clef
// ---------------------------------------------------------------------------
describe("actions.score.clef", () => {
  it("create calls invoke correctly", async () => {
    await actions.score.clef.create(1, 0, 100, 71, 0, ClefDrawType.G);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_clef_create", {
      flow_id: 1,
      tick: 0,
      stave_id: 100,
      pitch: 71,
      offset: 0,
      clef_draw_type: ClefDrawType.G,
      hidden: false,
    });
  });

  it("clear calls invoke correctly", async () => {
    await actions.score.clef.clear(1, 0, 100);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_clef_clear", {
      flow_id: 1,
      tick: 0,
      stave_id: 100,
    });
  });

  it("setHidden calls invoke correctly", async () => {
    await actions.score.clef.setHidden(1, 100, 9, true);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_clef_update", {
      flow_id: 1,
      stave_id: 100,
      clef_id: 9,
      hidden: true,
    });
  });
});

// ---------------------------------------------------------------------------
// score.keySignature
// ---------------------------------------------------------------------------
describe("actions.score.keySignature", () => {
  it("create calls invoke correctly", async () => {
    await actions.score.keySignature.create(1, 0, KeySignatureMode.Major, 0);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_key_signature_create",
      {
        flow_id: 1,
        tick: 0,
        mode: KeySignatureMode.Major,
        offset: 0,
        hidden: false,
      },
    );
  });

  it("clearAtTick calls invoke correctly", async () => {
    await actions.score.keySignature.clearAtTick(1, 0);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_key_signature_clear",
      { flow_id: 1, tick: 0 },
    );
  });

  it("setHidden calls invoke correctly", async () => {
    await actions.score.keySignature.setHidden(1, 12, true);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_key_signature_update",
      { flow_id: 1, key_signature_id: 12, hidden: true },
    );
  });
});

// ---------------------------------------------------------------------------
// score.timeSignature
// ---------------------------------------------------------------------------
describe("actions.score.timeSignature", () => {
  it("create calls invoke correctly", async () => {
    await actions.score.timeSignature.create(
      1,
      0,
      4,
      NoteDuration.Quarter,
      TimeSignatureDrawType.Regular,
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_time_signature_create",
      {
        flow_id: 1,
        tick: 0,
        beats: 4,
        beat_type: NoteDuration.Quarter,
        draw_type: TimeSignatureDrawType.Regular,
        hidden: false,
      },
    );
  });

  it("clearAtTick calls invoke correctly", async () => {
    await actions.score.timeSignature.clearAtTick(1, 0);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_time_signature_clear",
      { flow_id: 1, tick: 0 },
    );
  });

  it("update calls invoke correctly", async () => {
    await actions.score.timeSignature.update(
      1,
      9,
      TimeSignatureDrawType.CommonTime,
      true,
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_time_signature_update",
      {
        flow_id: 1,
        time_signature_id: 9,
        draw_type: TimeSignatureDrawType.CommonTime,
        hidden: true,
      },
    );
  });
});

// ---------------------------------------------------------------------------
// score.tone
// ---------------------------------------------------------------------------
describe("actions.score.tone", () => {
  it("create calls invoke and returns id", async () => {
    mockInvoke.mockResolvedValueOnce(200);
    const id = await actions.score.tone.create(
      10,
      32,
      32,
      60,
      Accidental.Natural,
      100,
      [Articulation.Staccato],
    );
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_create", {
      track_id: 10,
      tick: 32,
      duration: 32,
      pitch_int: 60,
      accidental: Accidental.Natural,
      velocity: 100,
      articulations: [Articulation.Staccato],
      tuplet_id: null,
    });
    expect(id).toBe(200);
  });

  it("create forwards optional tuplet context through the existing seam", async () => {
    mockInvoke.mockResolvedValueOnce(201);

    const id = await actions.score.tone.create(
      10,
      16,
      16,
      67,
      Accidental.Natural,
      90,
      [Articulation.Accent],
      44,
    );

    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_create", {
      track_id: 10,
      tick: 16,
      duration: 16,
      pitch_int: 67,
      accidental: Accidental.Natural,
      velocity: 90,
      articulations: [Articulation.Accent],
      tuplet_id: 44,
    });
    expect(id).toBe(201);
  });

  it("shift calls invoke correctly", async () => {
    await actions.score.tone.shift(10, 200, 64);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_shift", {
      track_id: 10,
      entry_id: 200,
      new_tick: 64,
      current_tuplet_id: null,
      target_tuplet_id: null,
    });
  });

  it("unifyArticulations calls invoke correctly", async () => {
    await actions.score.tone.unifyArticulations(10, 200);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_unify_articulations",
      { track_id: 10, entry_id: 200, tuplet_id: null },
    );
  });

  it("setDuration calls invoke correctly", async () => {
    await actions.score.tone.setDuration(10, 200, 32);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_duration_set", {
      track_id: 10,
      tone_id: 200,
      duration: 32,
      tuplet_id: null,
    });
  });

  it("setPositionAndDuration calls invoke correctly", async () => {
    await actions.score.tone.setPositionAndDuration(10, 200, 64, 16);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_set_position_and_duration",
      {
        track_id: 10,
        tone_id: 200,
        tick: 64,
        duration: 16,
        current_tuplet_id: null,
        target_tuplet_id: null,
      },
    );
  });

  it("setPitch calls invoke correctly", async () => {
    await actions.score.tone.setPitch(10, null, 200, 62, Accidental.Sharp);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_pitch_set", {
      track_id: 10,
      tone_id: 200,
      pitch_int: 62,
      accidental: Accidental.Sharp,
      tuplet_id: null,
    });
  });

  it("setPitch passes tuplet context when provided", async () => {
    await actions.score.tone.setPitch(10, 44, 200, 62, null);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_pitch_set", {
      track_id: 10,
      tone_id: 200,
      pitch_int: 62,
      accidental: null,
      tuplet_id: 44,
    });
  });

  it("remove calls invoke correctly", async () => {
    await actions.score.tone.remove(10, null, 200);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_duration_remove",
      { track_id: 10, tone_id: 200, tuplet_id: null },
    );
  });

  it("remove passes tuplet context when provided", async () => {
    await actions.score.tone.remove(10, 44, 200);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_duration_remove",
      { track_id: 10, tone_id: 200, tuplet_id: 44 },
    );
  });

  it("slice calls invoke correctly", async () => {
    await actions.score.tone.slice(10, 200, 48, null, null);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_slice", {
      track_id: 10,
      tone_id: 200,
      slice_tick: 48,
      current_tuplet_id: null,
      target_tuplet_id: null,
    });
  });

  it("slice passes ownership context when provided", async () => {
    await actions.score.tone.slice(10, 200, 16, 44, 44);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_slice", {
      track_id: 10,
      tone_id: 200,
      slice_tick: 16,
      current_tuplet_id: 44,
      target_tuplet_id: 44,
    });
  });

  it("slice supports cross-owner split context", async () => {
    await actions.score.tone.slice(10, 200, 96, 44, null);
    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tone_slice", {
      track_id: 10,
      tone_id: 200,
      slice_tick: 96,
      current_tuplet_id: 44,
      target_tuplet_id: null,
    });
  });

  it("shiftPitchByStep calls invoke correctly", async () => {
    await actions.score.tone.shiftPitchByStep(10, null, 200, 2);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_pitch_shift_by_steps",
      { track_id: 10, tone_id: 200, steps: 2, tuplet_id: null },
    );
  });

  it("shiftPitchByStep passes tuplet context when provided", async () => {
    await actions.score.tone.shiftPitchByStep(10, 44, 200, 2);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_pitch_shift_by_steps",
      { track_id: 10, tone_id: 200, steps: 2, tuplet_id: 44 },
    );
  });

  it("shiftPitchByOctave calls invoke correctly", async () => {
    await actions.score.tone.shiftPitchByOctave(10, null, 200, 1);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_pitch_shift_by_octaves",
      { track_id: 10, tone_id: 200, octaves: 1, tuplet_id: null },
    );
  });

  it("shiftPitchByOctave passes tuplet context when provided", async () => {
    await actions.score.tone.shiftPitchByOctave(10, 44, 200, 1);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_pitch_shift_by_octaves",
      { track_id: 10, tone_id: 200, octaves: 1, tuplet_id: 44 },
    );
  });

  it("setAccidental calls invoke correctly", async () => {
    await actions.score.tone.setAccidental(10, null, 200, Accidental.Flat);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_accidental_set",
      {
        track_id: 10,
        tone_id: 200,
        accidental: Accidental.Flat,
        tuplet_id: null,
      },
    );
  });

  it("setAccidental passes tuplet context when provided", async () => {
    await actions.score.tone.setAccidental(10, 44, 200, Accidental.Flat);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_accidental_set",
      {
        track_id: 10,
        tone_id: 200,
        accidental: Accidental.Flat,
        tuplet_id: 44,
      },
    );
  });

  it("setArticulation calls invoke correctly", async () => {
    await actions.score.tone.setArticulation(
      10,
      null,
      200,
      Articulation.Tenuto,
    );
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_articulation_set",
      {
        track_id: 10,
        tone_id: 200,
        articulation: Articulation.Tenuto,
        tuplet_id: null,
      },
    );
  });

  it("setArticulation passes tuplet context when provided", async () => {
    await actions.score.tone.setArticulation(10, 44, 200, Articulation.Tenuto);
    expect(mockInvoke).toHaveBeenCalledWith(
      "score_entries_tone_articulation_set",
      {
        track_id: 10,
        tone_id: 200,
        articulation: Articulation.Tenuto,
        tuplet_id: 44,
      },
    );
  });
});

describe("actions.score.tuplet", () => {
  it("create calls invoke and returns id", async () => {
    mockInvoke.mockResolvedValueOnce(300);

    const id = await actions.score.tuplet.create(
      10,
      32,
      3,
      2,
      NoteDuration.Eighth,
      [3],
    );

    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tuplet_create", {
      track_id: 10,
      tick: 32,
      note_count: 3,
      in_space_of: 2,
      base_note_value: NoteDuration.Eighth,
      groupings: [3],
    });
    expect(id).toBe(300);
  });

  it("remove calls invoke correctly", async () => {
    await actions.score.tuplet.remove(10, 300);

    expect(mockInvoke).toHaveBeenCalledWith("score_entries_tuplet_remove", {
      track_id: 10,
      tuplet_id: 300,
    });
  });
});

// ---------------------------------------------------------------------------
// transport
// ---------------------------------------------------------------------------
describe("actions.transport", () => {
  it("play calls invoke('transport_play')", async () => {
    await actions.transport.play();
    expect(mockInvoke).toHaveBeenCalledWith("transport_play");
  });

  it("pause calls invoke('transport_pause')", async () => {
    await actions.transport.pause();
    expect(mockInvoke).toHaveBeenCalledWith("transport_pause");
  });

  it("seek calls invoke with tick", async () => {
    await actions.transport.seek(128);
    expect(mockInvoke).toHaveBeenCalledWith("transport_seek", { tick: 128 });
  });

  it("getState calls invoke and returns the state", async () => {
    const mockState = { playing: true, current_tick: 64, tempo: 120 };
    mockInvoke.mockResolvedValueOnce(mockState);
    const state = await actions.transport.getState();
    expect(mockInvoke).toHaveBeenCalledWith("transport_get_state");
    expect(state).toEqual(mockState);
  });

  it("auditionTone calls invoke with the addressed instrument and pitch", async () => {
    await actions.transport.auditionTone(10, 64);

    expect(mockInvoke).toHaveBeenCalledWith("playback_audition_tone", {
      instrument_id: 10,
      pitch_int: 64,
    });
  });
});

// ---------------------------------------------------------------------------
// file
// ---------------------------------------------------------------------------
describe("actions.file", () => {
  describe("save — existing file path", () => {
    it("saves to the current path and adds to recents", async () => {
      mockInvoke.mockResolvedValueOnce("/project/song.soloc"); // get_current_file_path
      await actions.file.save();
      expect(mockInvoke).toHaveBeenCalledWith("save_project_to_file", {
        file_path: "/project/song.soloc",
      });
      expect(mockInvoke).toHaveBeenCalledWith("add_recent_file", {
        file_path: "/project/song.soloc",
      });
    });
  });

  describe("save — no existing file", () => {
    it("falls back to saveAs when no current path", async () => {
      mockInvoke.mockResolvedValueOnce(null); // get_current_file_path → null
      vi.mocked(save).mockResolvedValueOnce("/new/song.soloc");
      await actions.file.save();
      // save dialog should have been opened
      expect(save).toHaveBeenCalled();
      expect(mockInvoke).toHaveBeenCalledWith("save_project_to_file", {
        file_path: "/new/song.soloc",
      });
    });
  });

  describe("saveAs", () => {
    it("shows dialog and saves to chosen path", async () => {
      vi.mocked(save).mockResolvedValueOnce("/chosen/file.soloc");
      await actions.file.saveAs();
      expect(mockInvoke).toHaveBeenCalledWith("save_project_to_file", {
        file_path: "/chosen/file.soloc",
      });
      expect(mockInvoke).toHaveBeenCalledWith("add_recent_file", {
        file_path: "/chosen/file.soloc",
      });
    });

    it("throws when dialog is cancelled (returns null)", async () => {
      vi.mocked(save).mockResolvedValueOnce(null);
      await expect(actions.file.saveAs()).rejects.toThrow(
        "Save cancelled by user",
      );
    });
  });

  describe("load", () => {
    it("opens dialog and loads the chosen file", async () => {
      vi.mocked(open).mockResolvedValueOnce("/loaded/file.soloc");
      await actions.file.load();
      expect(mockInvoke).toHaveBeenCalledWith("load_project_from_file", {
        file_path: "/loaded/file.soloc",
      });
      expect(mockInvoke).toHaveBeenCalledWith("add_recent_file", {
        file_path: "/loaded/file.soloc",
      });
    });

    it("throws when dialog is cancelled (returns null)", async () => {
      vi.mocked(open).mockResolvedValueOnce(null);
      await expect(actions.file.load()).rejects.toThrow(
        "Load cancelled or invalid file selection",
      );
    });

    it("throws when dialog returns an array", async () => {
      // open() typed as single or array — array means multi-select which is invalid
      vi.mocked(open).mockResolvedValueOnce([
        "/a.soloc",
        "/b.soloc",
      ] as unknown as string);
      await expect(actions.file.load()).rejects.toThrow(
        "Load cancelled or invalid file selection",
      );
    });
  });

  describe("loadFromPath", () => {
    it("loads and adds to recents", async () => {
      await actions.file.loadFromPath("/recent/song.soloc");
      expect(mockInvoke).toHaveBeenCalledWith("load_project_from_file", {
        file_path: "/recent/song.soloc",
      });
      expect(mockInvoke).toHaveBeenCalledWith("add_recent_file", {
        file_path: "/recent/song.soloc",
      });
    });
  });
});
