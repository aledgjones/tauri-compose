import { describe, it, expect, vi } from "vitest";
import { screen, userEvent } from "../../services/test-utils";
import { renderWithProviders } from "../../services/test-utils";
import { Main } from "./index";

const { mockToggleDebug, mockLoadScoreFixture } = vi.hoisted(() => ({
  mockToggleDebug: vi.fn(),
  mockLoadScoreFixture: vi.fn(),
}));

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
    debug: {
      loadScoreFixture: mockLoadScoreFixture,
    },
    ui: {
      renderer: {
        toggleDebug: mockToggleDebug,
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

  it("loads the selected score fixture from the debugger", async () => {
    const user = userEvent.setup();
    renderWithProviders(<Main />);

    await user.click(screen.getByRole("button", { name: /^Load$/i }));

    expect(mockLoadScoreFixture).toHaveBeenCalledTimes(1);
  });
});
