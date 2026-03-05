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

  it("does not call setZoom when already at max zoom and incrementing", () => {
    const state = createAppState({ ui: { renderer: { zoom: 800 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    const buttons = screen.getAllByRole("button");
    const zoomInButton = buttons[buttons.length - 2];
    fireEvent.click(zoomInButton);
    expect(actions.ui.renderer.setZoom).not.toHaveBeenCalled();
  });

  it("does not call setZoom when already at min zoom and decrementing", () => {
    const state = createAppState({ ui: { renderer: { zoom: 25 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    const buttons = screen.getAllByRole("button");
    const zoomOutButton = buttons[buttons.length - 1];
    fireEvent.click(zoomOutButton);
    expect(actions.ui.renderer.setZoom).not.toHaveBeenCalled();
  });

  it("opens the preset dropdown when the zoom display button is clicked", () => {
    const state = createAppState({ ui: { renderer: { zoom: 100 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    const buttons = screen.getAllByRole("button");
    const toggleButton = buttons[0];
    fireEvent.click(toggleButton);
    // Preset options should now be visible (rendered as option roles)
    expect(screen.getAllByRole("option").length).toBeGreaterThan(0);
  });

  it("closes the dropdown and calls setZoom when a preset is selected", () => {
    const state = createAppState({ ui: { renderer: { zoom: 100 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    // Open dropdown
    fireEvent.click(screen.getAllByRole("button")[0]);
    // Click the 50% preset option
    const option50 = screen.getByRole("option", { name: "50%" });
    fireEvent.click(option50);
    expect(actions.ui.renderer.setZoom).toHaveBeenCalledWith(50);
    // Dropdown should be closed — options no longer visible
    expect(screen.queryAllByRole("option")).toHaveLength(0);
  });

  it("marks the current zoom preset as selected in the dropdown", () => {
    const state = createAppState({ ui: { renderer: { zoom: 150 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    // Open dropdown
    fireEvent.click(screen.getAllByRole("button")[0]);
    const selected = screen.getByRole("option", { name: "150%" });
    expect(selected).toHaveAttribute("aria-selected", "true");
  });

  it("closes the dropdown when clicking outside", () => {
    const state = createAppState({ ui: { renderer: { zoom: 100 } } as never });
    renderWithProviders(<ZoomWidget />, { appState: state });
    // Open dropdown
    fireEvent.click(screen.getAllByRole("button")[0]);
    expect(screen.getAllByRole("option").length).toBeGreaterThan(0);
    // Click outside
    fireEvent.mouseDown(document.body);
    expect(screen.queryAllByRole("option")).toHaveLength(0);
  });
});
