import { describe, it, expect, vi, beforeEach } from "vitest";
import { act, render, screen, waitFor } from "@testing-library/react";
import { AppStateProvider } from "./provider";
import { initialAppState } from "./context";
import { AppState, NoteDuration } from "../types";

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
    listenMock.mockReset();
    listenMock.mockResolvedValue(vi.fn());
    getStateMock.mockClear();
    getStateMock.mockResolvedValue({
      playing: false,
      current_tick: 0,
      tempo: 120,
    });
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
});
