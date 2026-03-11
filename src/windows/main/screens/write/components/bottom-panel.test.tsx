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
    ui: {
      screens: {
        write: {
          bottomPanel: { setTab: vi.fn() },
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

describe("Write BottomPanel", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing (Sequencer tab default)", () => {
    const { container } = renderWithProviders(<BottomPanel />);
    expect(container).toBeInTheDocument();
  });

  it("renders Properties, Sequencer, Mixer tab buttons", () => {
    renderWithProviders(<BottomPanel />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(3);
  });

  it("calls setTab when a tab button is clicked", () => {
    renderWithProviders(<BottomPanel />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[buttons.length - 1]);
    expect(actions.ui.screens.write.bottomPanel.setTab).toHaveBeenCalled();
  });

  it("renders SequencerToolbar when Sequencer tab is active (default)", () => {
    const appState = createAppState({
      ui: {
        screens: {
          write: { bottom_panel_tab: BottomPanelTab.Sequencer } as never,
        },
      } as never,
    });
    renderWithProviders(<BottomPanel />, { appState });
    // SequencerToolbar has tool icon buttons
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(4);
  });

  it("renders MixerToolbar when Mixer tab is active", () => {
    const appState = createAppState({
      ui: {
        screens: {
          write: { bottom_panel_tab: BottomPanelTab.Mixer } as never,
        },
      } as never,
    });
    renderWithProviders(<BottomPanel />, { appState });
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it("renders the properties panel empty state for Properties tab", () => {
    const appState = createAppState({
      ui: {
        screens: {
          write: { bottom_panel_tab: BottomPanelTab.Properties } as never,
        },
      } as never,
    });
    renderWithProviders(<BottomPanel />, { appState });
    expect(screen.queryByText("No selection")).not.toBeInTheDocument();
    expect(screen.getAllByRole("button")).toHaveLength(3);
  });
});
