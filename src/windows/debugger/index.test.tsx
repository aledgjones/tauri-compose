import { beforeEach, describe, expect, it, vi } from "vitest";
import { screen } from "../../services/test-utils";
import { renderWithProviders } from "../../services/test-utils";
import { Main } from "./index";

const { mockToggleDebug } = vi.hoisted(() => ({
  mockToggleDebug: vi.fn(),
}));

const { mockObjectInspector } = vi.hoisted(() => ({
  mockObjectInspector: vi.fn((_props: unknown) => (
    <div data-testid="object-inspector" />
  )),
}));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ show: vi.fn(), setTitle: vi.fn() }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock("react-inspector", () => ({
  ObjectInspector: mockObjectInspector,
  chromeDark: {},
}));

vi.mock("../../services/actions", () => ({
  actions: {
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
  beforeEach(() => {
    mockObjectInspector.mockClear();
    mockToggleDebug.mockClear();
  });

  it("renders the app bar drag region", () => {
    const { container } = renderWithProviders(<Main />);
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("renders the timing and debug surfaces", () => {
    renderWithProviders(<Main />);

    expect(screen.getByTestId("performance-timings")).toBeInTheDocument();
    expect(screen.getByText("Render Spacing Debugging")).toBeInTheDocument();
    expect(screen.getAllByTestId("object-inspector")).toHaveLength(2);
  });

  it("does not render the retired VST probe workflow", () => {
    renderWithProviders(<Main />);

    expect(
      screen.queryByRole("button", { name: /Run Load\/Init Probe/i }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/Persisted Probe Log/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/Probe Profile:/i)).not.toBeInTheDocument();
  });
});
