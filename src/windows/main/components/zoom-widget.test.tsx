import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { ZoomWidget } from "./zoom-widget";
import {
  renderWithProviders,
  createAppState,
} from "../../../services/test-utils";

vi.mock("../../../services/actions", () => ({
  actions: {
    ui: {
      renderer: {
        setZoom: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

import { actions } from "../../../services/actions";

describe("ZoomWidget", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders current zoom percentage", () => {
    const state = createAppState({ ui: { renderer: { zoom: 100 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    expect(screen.getByText("100%")).toBeInTheDocument();
  });

  it("renders at different zoom level", () => {
    const state = createAppState({ ui: { renderer: { zoom: 150 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    expect(screen.getByText("150%")).toBeInTheDocument();
  });

  it("calls setZoom with increased value on zoom-in button click", () => {
    // At zoom=100, next preset is 125
    const state = createAppState({ ui: { renderer: { zoom: 100 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    const buttons = screen.getAllByRole("button");
    // zoom-in button is the second to last (after the dropdown toggle)
    const zoomInButton = buttons[buttons.length - 2];
    fireEvent.click(zoomInButton);
    expect(actions.ui.renderer.setZoom).toHaveBeenCalledWith(125);
  });

  it("calls setZoom with decreased value on zoom-out button click", () => {
    // At zoom=100, previous preset is 75
    const state = createAppState({ ui: { renderer: { zoom: 100 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    const buttons = screen.getAllByRole("button");
    const zoomOutButton = buttons[buttons.length - 1];
    fireEvent.click(zoomOutButton);
    expect(actions.ui.renderer.setZoom).toHaveBeenCalledWith(75);
  });
});
