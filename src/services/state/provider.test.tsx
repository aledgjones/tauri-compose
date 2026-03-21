import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import { AppStateProvider } from "./provider";
import { initialAppState } from "./context";
import { AppState, NoteDuration, PluginInventoryState } from "../types";
import { invoke } from "@tauri-apps/api/core";
import { actions } from "../actions";

const { listenMock, getStateMock } = vi.hoisted(() => ({
  listenMock: vi.fn(),
  getStateMock: vi.fn().mockResolvedValue({
    playing: false,
    current_tick: 0,
    tempo: 120,
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((p: string) => p),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: vi.fn().mockImplementation(() => ({
    setFocus: vi.fn(),
    once: vi.fn().mockResolvedValue(vi.fn()),
    close: vi.fn(),
  })),
}));

vi.mock("../actions", () => ({
  actions: {
    initialize: vi.fn().mockResolvedValue(undefined),
    transport: {
      play: vi.fn().mockResolvedValue(undefined),
      pause: vi.fn().mockResolvedValue(undefined),
      stop: vi.fn().mockResolvedValue(undefined),
      seek: vi.fn().mockResolvedValue(undefined),
      getState: getStateMock,
    },
    file: {
      save: vi.fn().mockResolvedValue(undefined),
      saveAs: vi.fn().mockResolvedValue(undefined),
      load: vi.fn().mockResolvedValue(undefined),
      loadFromPath: vi.fn().mockResolvedValue(undefined),
    },
    reset: vi.fn().mockResolvedValue(undefined),
  },
}));

describe("AppStateProvider", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useRealTimers();
    vi.stubGlobal("alert", vi.fn());
    listenMock.mockReset();
    listenMock.mockResolvedValue(vi.fn());
    getStateMock.mockClear();
    getStateMock.mockResolvedValue({
      playing: false,
      current_tick: 0,
      tempo: 120,
    });
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("renders children", async () => {
    await act(async () => {
      render(
        <AppStateProvider>
          <div>child content</div>
        </AppStateProvider>,
      );
    });
    expect(screen.getByText("child content")).toBeInTheDocument();
  });

  it("renders multiple children", async () => {
    await act(async () => {
      render(
        <AppStateProvider>
          <span>first</span>
          <span>second</span>
        </AppStateProvider>,
      );
    });
    expect(screen.getByText("first")).toBeInTheDocument();
    expect(screen.getByText("second")).toBeInTheDocument();
  });

  it("renders with isMainWindow=false (default)", async () => {
    await act(async () => {
      render(
        <AppStateProvider isMainWindow={false}>
          <div>non-main</div>
        </AppStateProvider>,
      );
    });
    expect(screen.getByText("non-main")).toBeInTheDocument();
  });

  it("renders with isMainWindow=true", async () => {
    await act(async () => {
      render(
        <AppStateProvider isMainWindow={true}>
          <div>main-window-child</div>
        </AppStateProvider>,
      );
    });
    expect(screen.getByText("main-window-child")).toBeInTheDocument();
  });

  it("refreshes idle transport state when tempo data changes", async () => {
    let generalListener:
      | ((event: { payload: typeof initialAppState }) => void)
      | null = null;

    listenMock.mockImplementation(async (eventName, callback) => {
      if (eventName === "soloc://state/general") {
        generalListener = callback as (event: {
          payload: typeof initialAppState;
        }) => void;
      }

      return vi.fn();
    });

    await act(async () => {
      render(
        <AppStateProvider>
          <div>child content</div>
        </AppStateProvider>,
      );
    });

    await waitFor(() => {
      expect(getStateMock).toHaveBeenCalledTimes(1);
    });

    const updatedAppState: AppState = {
      ...initialAppState,
      ui: {
        ...initialAppState.ui,
        flow_id: 1,
      },
      score: {
        ...initialAppState.score,
        flows: {
          order: [1],
          by_id: {
            1: {
              id: 1,
              title: "Flow 1",
              player_ids: [],
              length: 128,
              master_id: 10,
              staves: { by_id: {} },
            },
          },
        },
        tracks: {
          by_id: {
            10: {
              id: 10,
              entries: {
                by_tick: { 0: [100] },
                by_id: {
                  100: {
                    id: 100,
                    tick: 0,
                    type: "Tempo",
                    text: "",
                    beat_type: NoteDuration.Quarter,
                    dots: 0,
                    bpm: 96,
                    hidden: false,
                    show_text: false,
                    show_metronome: true,
                    parenthesized: false,
                  },
                },
              },
            },
          },
        },
      },
    };

    await act(async () => {
      generalListener?.({ payload: updatedAppState });
    });

    await waitFor(() => {
      expect(getStateMock).toHaveBeenCalledTimes(2);
    });
  });

  it("refreshes idle transport state when the active flow changes between multiple flows", async () => {
    let generalListener:
      | ((event: { payload: typeof initialAppState }) => void)
      | null = null;

    listenMock.mockImplementation(async (eventName, callback) => {
      if (eventName === "soloc://state/general") {
        generalListener = callback as (event: {
          payload: typeof initialAppState;
        }) => void;
      }

      return vi.fn();
    });

    await act(async () => {
      render(
        <AppStateProvider>
          <div>child content</div>
        </AppStateProvider>,
      );
    });

    await waitFor(() => {
      expect(getStateMock).toHaveBeenCalledTimes(1);
    });

    const appStateForFlow = (
      flowId: number,
      tempoId: number,
      bpm: number,
    ): AppState => ({
      ...initialAppState,
      ui: {
        ...initialAppState.ui,
        flow_id: flowId,
      },
      score: {
        ...initialAppState.score,
        flows: {
          order: [1, 2],
          by_id: {
            1: {
              id: 1,
              title: "Verse",
              player_ids: [],
              length: 128,
              master_id: 10,
              staves: { by_id: {} },
            },
            2: {
              id: 2,
              title: "Chorus",
              player_ids: [],
              length: 128,
              master_id: 20,
              staves: { by_id: {} },
            },
          },
        },
        tracks: {
          by_id: {
            10: {
              id: 10,
              entries: {
                by_tick: { 0: [100] },
                by_id: {
                  100: {
                    id: 100,
                    tick: 0,
                    type: "Tempo",
                    text: "",
                    beat_type: NoteDuration.Quarter,
                    dots: 0,
                    bpm: 96,
                    hidden: false,
                    show_text: false,
                    show_metronome: true,
                    parenthesized: false,
                  },
                },
              },
            },
            20: {
              id: 20,
              entries: {
                by_tick: { 0: [tempoId] },
                by_id: {
                  [tempoId]: {
                    id: tempoId,
                    tick: 0,
                    type: "Tempo",
                    text: "",
                    beat_type: NoteDuration.Quarter,
                    dots: 0,
                    bpm,
                    hidden: false,
                    show_text: false,
                    show_metronome: true,
                    parenthesized: false,
                  },
                },
              },
            },
          },
        },
      },
    });

    await act(async () => {
      generalListener?.({ payload: appStateForFlow(1, 200, 96) });
    });

    await waitFor(() => {
      expect(getStateMock).toHaveBeenCalledTimes(2);
    });

    await act(async () => {
      generalListener?.({ payload: appStateForFlow(2, 201, 132) });
    });

    await waitFor(() => {
      expect(getStateMock).toHaveBeenCalledTimes(3);
    });
  });

  it("wires main-window file menu events and refreshes menus after init and recents changes", async () => {
    vi.useFakeTimers();

    const listeners = new Map<string, (event: { payload: string }) => void>();
    listenMock.mockImplementation(async (eventName, callback) => {
      listeners.set(
        eventName,
        callback as (event: { payload: string }) => void,
      );
      return vi.fn();
    });

    const invokeMock = vi.mocked(invoke);

    await act(async () => {
      render(
        <AppStateProvider isMainWindow={true}>
          <div>main-window-child</div>
        </AppStateProvider>,
      );
      await Promise.resolve();
    });

    expect(listeners.has("soloc://menu/save")).toBe(true);
    expect(listeners.has("soloc://menu/save-as")).toBe(true);
    expect(listeners.has("soloc://menu/open")).toBe(true);
    expect(listeners.has("soloc://menu/open-recent")).toBe(true);
    expect(listeners.has("soloc://menu/new")).toBe(true);
    expect(listeners.has("soloc://recents-changed")).toBe(true);

    await act(async () => {
      await Promise.resolve();
      await vi.advanceTimersByTimeAsync(100);
      await Promise.resolve();
    });

    expect(invokeMock).toHaveBeenCalledWith("refresh_menus");

    await act(async () => {
      listeners.get("soloc://menu/save")?.({ payload: "" });
      listeners.get("soloc://menu/save-as")?.({ payload: "" });
      listeners.get("soloc://menu/open")?.({ payload: "" });
      listeners.get("soloc://menu/open-recent")?.({
        payload: "/Users/aledjones/example.soloc",
      });
      listeners.get("soloc://menu/new")?.({ payload: "" });
      listeners.get("soloc://recents-changed")?.({ payload: "" });
      await Promise.resolve();
    });

    expect(actions.file.save).toHaveBeenCalledTimes(1);
    expect(actions.file.saveAs).toHaveBeenCalledTimes(1);
    expect(actions.file.load).toHaveBeenCalledTimes(1);
    expect(actions.file.loadFromPath).toHaveBeenCalledWith(
      "/Users/aledjones/example.soloc",
    );
    expect(actions.reset).toHaveBeenCalledTimes(1);

    expect(invokeMock).toHaveBeenCalledWith("refresh_menus");
  });

  it("does not register main-window menu listeners for secondary windows", async () => {
    const listeners = new Set<string>();
    listenMock.mockImplementation(async (eventName) => {
      listeners.add(eventName);
      return vi.fn();
    });

    await act(async () => {
      render(
        <AppStateProvider isMainWindow={false}>
          <div>secondary-window-child</div>
        </AppStateProvider>,
      );
    });

    expect(listeners.has("soloc://state/general")).toBe(true);
    expect(listeners.has("soloc://state/audio")).toBe(true);
    expect(listeners.has("soloc://state/sampler")).toBe(true);
    expect(listeners.has("soloc://menu/save")).toBe(false);
    expect(listeners.has("soloc://menu/open")).toBe(false);
    expect(listeners.has("soloc://menu/open-recent")).toBe(false);
    expect(listeners.has("soloc://recents-changed")).toBe(false);
  });

  it("alerts once when plugin inventory errors are emitted", async () => {
    let pluginInventoryListener:
      | ((event: { payload: PluginInventoryState }) => void)
      | null = null;

    listenMock.mockImplementation(async (eventName, callback) => {
      if (eventName === "soloc://state/plugin_inventory") {
        pluginInventoryListener = callback as (event: {
          payload: PluginInventoryState;
        }) => void;
      }

      return vi.fn();
    });

    await act(async () => {
      render(
        <AppStateProvider>
          <div>child content</div>
        </AppStateProvider>,
      );
    });

    const payload: PluginInventoryState = {
      entries: [],
      errors: [
        {
          kind: "storage_error",
          message:
            "Failed to open plugin inventory database '/tmp/plugin_inventory': nope",
        },
      ],
      last_discovered_at_ms: null,
    };

    await act(async () => {
      pluginInventoryListener?.({ payload });
    });

    expect(globalThis.alert).toHaveBeenCalledTimes(1);
    expect(globalThis.alert).toHaveBeenCalledWith(
      expect.stringContaining("Failed to open plugin inventory database"),
    );

    await act(async () => {
      pluginInventoryListener?.({ payload });
    });

    expect(globalThis.alert).toHaveBeenCalledTimes(1);
  });
});
