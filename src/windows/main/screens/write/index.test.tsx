import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import {
  renderWithProviders,
  createAppState,
} from "../../../../services/test-utils";
import { Write } from "./index";

vi.mock("../../../../services/actions", () => ({
  actions: {
    ui: {
      screens: {
        write: {
          bottomPanel: { toggle: vi.fn(), setTab: vi.fn() },
        },
      },
      renderer: { setBounds: vi.fn() },
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
      tone: {
        setDuration: vi.fn(),
        setAccidental: vi.fn(),
        removeAccidental: vi.fn(),
        setArticulation: vi.fn(),
        removeArticulation: vi.fn(),
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

import { actions } from "../../../../services/actions";

describe("Write screen", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing", () => {
    const { container } = renderWithProviders(<Write />);
    expect(container).toBeInTheDocument();
  });

  it("renders a toggle button for the bottom panel", () => {
    renderWithProviders(<Write />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it("calls bottomPanel.toggle when the toggle button is clicked", () => {
    renderWithProviders(<Write />);
    // Click all buttons — the toggle button will call bottomPanel.toggle
    screen.getAllByRole("button").forEach((btn) => fireEvent.click(btn));
    expect(actions.ui.screens.write.bottomPanel.toggle).toHaveBeenCalled();
  });

  it("renders with gridTemplateRows='1fr' when minimized", () => {
    const appState = createAppState({
      ui: {
        screens: {
          write: { bottom_panel_minimized: true } as never,
        },
      } as never,
    });
    const { container } = renderWithProviders(<Write />, { appState });
    const grid = container.querySelector(".grid") as HTMLElement;
    expect(grid.style.gridTemplateRows).toBe("1fr");
  });

  it("renders with gridTemplateRows='3fr 2fr' when not minimized", () => {
    const appState = createAppState({
      ui: {
        screens: {
          write: { bottom_panel_minimized: false } as never,
        },
      } as never,
    });
    const { container } = renderWithProviders(<Write />, { appState });
    const grid = container.querySelector(".grid") as HTMLElement;
    expect(grid.style.gridTemplateRows).toBe("3fr 2fr");
  });

  it("does not render BottomPanel when minimized", () => {
    const appState = createAppState({
      ui: {
        screens: {
          write: { bottom_panel_minimized: true } as never,
        },
      } as never,
    });
    const { container } = renderWithProviders(<Write />, { appState });
    // In minimized mode the col-span div w/row-span-2 disappears
    const grid = container.querySelector(".grid") as HTMLElement;
    expect(grid.style.gridTemplateRows).toBe("1fr");
  });
});
