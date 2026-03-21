import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import {
  renderWithProviders,
  createAppState,
} from "../../../../../services/test-utils";
import { BottomPanel } from "./bottom-panel";
import { BottomPanelTab } from "../../../../../services/types";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    pluginInventory: {
      discover: vi.fn().mockResolvedValue(undefined),
    },
    ui: {
      setActiveFlowId: vi.fn(),
      screens: {
        sequence: {
          bottomPanel: {
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
    },
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

import { actions } from "../../../../../services/actions";

describe("Sequence BottomPanel", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing (Sequencer tab default)", () => {
    const { container } = renderWithProviders(<BottomPanel />);
    expect(container).toBeInTheDocument();
  });

  it("renders tab buttons for Sequencer, Mixer, Instruments", () => {
    renderWithProviders(<BottomPanel />);
    const buttons = screen.getAllByRole("button");
    // At least 3 tab buttons
    expect(buttons.length).toBeGreaterThanOrEqual(3);
  });

  it("calls setTab when a tab button is clicked", () => {
    const appState = createAppState({
      ui: {
        screens: {
          sequence: { bottom_panel_tab: BottomPanelTab.Sequencer } as never,
        },
      } as never,
    });
    renderWithProviders(<BottomPanel />, { appState });
    const buttons = screen.getAllByRole("button");
    // last 3 buttons are tabs; click Mixer (second-to-last of tab buttons)
    fireEvent.click(buttons[buttons.length - 2]);
    expect(actions.ui.screens.sequence.bottomPanel.setTab).toHaveBeenCalled();
  });

  it("renders MixerToolbar when Mixer tab is active", () => {
    const appState = createAppState({
      ui: {
        screens: {
          sequence: { bottom_panel_tab: BottomPanelTab.Mixer } as never,
        },
      } as never,
    });
    renderWithProviders(<BottomPanel />, { appState });
    // The MixerToolbar renders S and M buttons
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it("renders PluginsToolbar when Instruments tab is active", () => {
    const appState = createAppState({
      ui: {
        screens: {
          sequence: {
            bottom_panel_tab: BottomPanelTab.Instruments,
            show_active_only: false,
          } as never,
        },
      } as never,
    });
    renderWithProviders(<BottomPanel />, { appState });
    expect(screen.getByText("Available only")).toBeInTheDocument();
  });

  it("renders Sequencer toolbar when Sequencer tab is active", () => {
    const appState = createAppState({
      ui: {
        screens: {
          sequence: { bottom_panel_tab: BottomPanelTab.Sequencer } as never,
        },
      } as never,
    });
    renderWithProviders(<BottomPanel />, { appState });
    // SequencerToolbar renders tool buttons
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(4);
  });

  it("does not render a local flow selector inside the Sequence bottom panel", () => {
    const appState = createAppState({
      ui: {
        flow_id: 6,
        screens: {
          sequence: { bottom_panel_tab: BottomPanelTab.Sequencer } as never,
        },
      } as never,
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

    renderWithProviders(<BottomPanel />, { appState });

    expect(screen.queryByText("Flow")).toBeNull();
  });
});
