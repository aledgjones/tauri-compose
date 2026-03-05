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
});
