import { describe, it, expect, vi } from "vitest";
import { screen } from "@testing-library/react";
import { renderWithProviders } from "../../services/test-utils";
import { Main } from "./index";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ show: vi.fn(), setTitle: vi.fn() }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock("react-inspector", () => ({
  ObjectInspector: () => <div data-testid="object-inspector" />,
  chromeDark: {},
}));

vi.mock("../../services/actions", () => ({
  actions: {
    ui: {
      renderer: {
        toggleDebug: vi.fn(),
      },
    },
  },
}));

vi.mock("./components/performance-timings", () => ({
  PerformanceTimings: () => <div data-testid="performance-timings" />,
}));

describe("Debugger Main", () => {
  it("renders the app bar drag region", () => {
    const { container } = renderWithProviders(<Main />);
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("renders the toggle debug switch", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("Render Spacing Debugging")).toBeInTheDocument();
  });
});
