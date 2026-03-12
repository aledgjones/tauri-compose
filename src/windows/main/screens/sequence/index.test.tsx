import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import {
  renderWithProviders,
  createAppState,
} from "../../../../services/test-utils";
import { Sequence } from "./index";

vi.mock("../../../../services/actions", () => ({
  actions: {
    ui: {
      setActiveFlowId: vi.fn(),
      screens: {
        sequence: {
          bottomPanel: {
            toggle: vi.fn(),
            setTab: vi.fn(),
            instruments: { setShowActiveOnly: vi.fn() },
          },
        },
      },
      sequencer: {
        setTool: vi.fn(),
        setSnap: vi.fn(),
        setCoords: vi.fn(),
        setInstrument: vi.fn(),
        setTrack: vi.fn(),
      },
    },
    score: {
      instrument: {
        setSolo: vi.fn(),
        setMute: vi.fn(),
        setVolume: vi.fn(),
      },
      flow: { assignPlayer: vi.fn(), unassignPlayer: vi.fn() },
    },
    sampler: { rescanPlugins: vi.fn().mockResolvedValue(undefined) },
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((p: string) => p),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: vi.fn().mockImplementation(() => ({
    setFocus: vi.fn(),
    once: vi.fn().mockResolvedValue(vi.fn()),
  })),
}));

import { actions } from "../../../../services/actions";

describe("Sequence screen", () => {
  beforeEach(() => vi.clearAllMocks());

  const buildMultiFlowState = () =>
    createAppState({
      ui: { flow_id: 6 } as never,
      score: {
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

  it("renders without crashing (default state — no flow selected)", () => {
    const { container } = renderWithProviders(<Sequence />);
    expect(container).toBeInTheDocument();
  });

  it("renders a toggle button for the bottom panel", () => {
    renderWithProviders(<Sequence />);
    // The chevron button is present regardless of state
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it("calls bottomPanel.toggle when the toggle button is clicked", () => {
    renderWithProviders(<Sequence />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[buttons.length - 1]);
    expect(actions.ui.screens.sequence.bottomPanel.toggle).toHaveBeenCalled();
  });

  it("does not render BottomPanel section when minimized", () => {
    const appState = createAppState({
      ui: {
        screens: {
          sequence: { bottom_panel_minimized: true } as never,
        },
      } as never,
    });
    const { container } = renderWithProviders(<Sequence />, { appState });
    // gridTemplateRows should be "1fr" only — BottomPanel not rendered
    const grid = container.querySelector(".grid") as HTMLElement;
    expect(grid.style.gridTemplateRows).toBe("1fr");
  });

  it("renders BottomPanel section when not minimized", () => {
    const appState = createAppState({
      ui: {
        screens: {
          sequence: { bottom_panel_minimized: false } as never,
        },
      } as never,
    });
    const { container } = renderWithProviders(<Sequence />, { appState });
    const grid = container.querySelector(".grid") as HTMLElement;
    expect(grid.style.gridTemplateRows).toBe("3fr 2fr");
  });

  it("renders the flow picker within the Sequence overview when multiple flows exist", () => {
    renderWithProviders(<Sequence />, { appState: buildMultiFlowState() });

    expect(
      screen.getByRole("button", { name: "2. Flow B" }),
    ).toBeInTheDocument();
  });

  it("changes the active flow from the persistent Sequence header", () => {
    renderWithProviders(<Sequence />, { appState: buildMultiFlowState() });

    fireEvent.click(screen.getByRole("button", { name: "2. Flow B" }));
    fireEvent.click(screen.getByRole("option", { name: "1. Flow A" }));

    expect(actions.ui.setActiveFlowId).toHaveBeenCalledWith(5);
  });
});
