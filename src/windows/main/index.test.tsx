import { describe, it, expect, vi } from "vitest";
import { screen, within } from "@testing-library/react";
import { renderWithProviders, createAppState } from "../../services/test-utils";
import { Main } from "./index";
import { Tab } from "../../services/types";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ show: vi.fn(), setTitle: vi.fn() }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((p: string) => p),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: vi.fn().mockImplementation(() => ({
    setFocus: vi.fn(),
    once: vi.fn(),
    close: vi.fn(),
  })),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
  emit: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
  open: vi.fn(),
}));

vi.mock("sortablejs", () => ({
  default: { create: vi.fn().mockReturnValue({ destroy: vi.fn() }) },
}));

vi.mock("../../services/actions", () => ({
  actions: {
    ui: {
      setTab: vi.fn(),
      setActiveLayoutId: vi.fn(),
      setActiveFlowId: vi.fn(),
      sequencer: {
        setTool: vi.fn(),
        setSnap: vi.fn(),
        setInstrument: vi.fn(),
        setTrack: vi.fn(),
      },
      renderer: {
        setBounds: vi.fn(),
        toggleDebug: vi.fn(),
      },
      screens: {
        sequence: {
          bottomPanel: {
            toggle: vi.fn(),
            setTab: vi.fn(),
            instruments: { setShowActiveOnly: vi.fn() },
          },
        },
      },
    },
    score: {
      layout: { create: vi.fn(), select: vi.fn() },
      flow: {
        create: vi.fn(),
        select: vi.fn(),
        rename: vi.fn(),
        remove: vi.fn(),
        assignPlayer: vi.fn(),
        unassignPlayer: vi.fn(),
      },
      player: { create: vi.fn(), select: vi.fn() },
      instrument: { setSolo: vi.fn(), setMute: vi.fn(), setVolume: vi.fn() },
    },
    transport: {
      play: vi.fn(),
      stop: vi.fn(),
      rewind: vi.fn(),
    },
  },
}));

describe("Main Window", () => {
  it("renders the app bar drag region", () => {
    const { container } = renderWithProviders(<Main />);
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("renders all tab options", () => {
    renderWithProviders(<Main />);
    expect(screen.getByRole("button", { name: "Setup" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Write" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Engrave" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Sequence" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Print" })).toBeInTheDocument();
  });

  it("exposes the flow switcher only inside the Sequence workflow", () => {
    const appState = createAppState({
      ui: { tab: Tab.Sequence, flow_id: 6 } as never,
      score: {
        layouts: {
          order: [1],
          by_id: {
            1: {
              id: 1,
              title: "Full Score",
              layout_type: 0,
              player_ids: [],
              flow_ids: [5, 6],
            },
          },
        },
        flows: {
          order: [5, 6],
          by_id: {
            5: {
              id: 5,
              title: "Flow A",
              player_ids: [],
              length: 512,
              master_id: 15,
              staves: { by_id: {} },
            },
            6: {
              id: 6,
              title: "Flow B",
              player_ids: [],
              length: 512,
              master_id: 16,
              staves: { by_id: {} },
            },
          },
        },
        tracks: {
          by_id: {
            15: { id: 15, entries: { by_tick: {}, by_id: {} } },
            16: { id: 16, entries: { by_tick: {}, by_id: {} } },
          },
        },
      } as never,
    });

    renderWithProviders(<Main />, { appState });

    expect(screen.getAllByRole("button", { name: "2. Flow B" })).toHaveLength(
      1,
    );
  });

  it("keeps workflow-specific flow switching out of the shared top bar", () => {
    const appState = createAppState({
      ui: { tab: Tab.Write, flow_id: 6 } as never,
      score: {
        layouts: {
          order: [1],
          by_id: {
            1: {
              id: 1,
              title: "Full Score",
              layout_type: 0,
              player_ids: [],
              flow_ids: [5, 6],
            },
          },
        },
        flows: {
          order: [5, 6],
          by_id: {
            5: {
              id: 5,
              title: "Flow A",
              player_ids: [],
              length: 512,
              master_id: 15,
              staves: { by_id: {} },
            },
            6: {
              id: 6,
              title: "Flow B",
              player_ids: [],
              length: 512,
              master_id: 16,
              staves: { by_id: {} },
            },
          },
        },
        tracks: {
          by_id: {
            15: { id: 15, entries: { by_tick: {}, by_id: {} } },
            16: { id: 16, entries: { by_tick: {}, by_id: {} } },
          },
        },
      } as never,
    });

    const { container } = renderWithProviders(<Main />, { appState });

    const topBar = container.querySelector(
      ".flex.items-center.p-2.bg-neutral-900",
    );

    expect(screen.getByRole("button", { name: "Write" })).toBeInTheDocument();
    expect(topBar).toBeInTheDocument();
    expect(
      within(topBar as HTMLElement).queryByRole("button", {
        name: "1. Flow A",
      }),
    ).not.toBeInTheDocument();
    expect(
      within(topBar as HTMLElement).queryByRole("button", {
        name: "2. Flow B",
      }),
    ).not.toBeInTheDocument();
  });
});
