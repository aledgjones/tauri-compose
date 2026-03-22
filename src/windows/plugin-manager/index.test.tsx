import { describe, it, expect, vi, beforeEach } from "vitest";
import { fireEvent, screen } from "@testing-library/react";
import { renderWithProviders } from "../../services/test-utils";
import { Main } from "./index";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: vi.fn(() => ({
    show: vi.fn().mockResolvedValue(undefined),
    setTitle: vi.fn().mockResolvedValue(undefined),
  })),
}));

const makePlugin = (overrides = {}) => ({
  id: "vst3:/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
  name: "Grand Piano",
  bundle_path: "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
  binary_path:
    "/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3/Contents/MacOS/Grand Piano",
  source_root: "/Library/Audio/Plug-Ins/VST3",
  availability: "available" as const,
  validation_failure: null,
  ...overrides,
});

describe("PluginManager Main", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders plugin inventory entries from shared provider state", () => {
    renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [makePlugin()],
      },
    });

    expect(screen.queryByText("Plugin inventory")).not.toBeInTheDocument();
    expect(
      screen.getByPlaceholderText("Search plugins..."),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Refresh plugin inventory" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
    expect(
      screen.getByText(/Contents\/MacOS\/Grand Piano/),
    ).toBeInTheDocument();
    expect(screen.queryByText("Root")).not.toBeInTheDocument();
    expect(screen.queryByText("Binary")).not.toBeInTheDocument();
  });

  it("reuses Available only filtering in the dedicated window", () => {
    renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [
          makePlugin(),
          makePlugin({
            id: "vst3:/Library/Audio/Plug-Ins/VST3/Broken.vst3",
            name: "Broken Plugin",
            availability: "unavailable",
            binary_path: null,
            validation_failure: {
              kind: "missing_bundle_binary",
              message: "Missing Contents/MacOS binary",
            },
          }),
        ],
      },
    });

    fireEvent.click(screen.getByRole("switch"));

    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
    expect(screen.queryByText("Broken Plugin")).not.toBeInTheDocument();
  });

  it("shows compact failure detail for unavailable plugins", () => {
    renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [
          makePlugin({
            id: "vst3:/Library/Audio/Plug-Ins/VST3/Broken.vst3",
            name: "Broken Plugin",
            availability: "unavailable",
            binary_path: null,
            validation_failure: {
              kind: "missing_bundle_binary",
              message: "Missing Contents/MacOS binary",
            },
          }),
        ],
      },
    });

    expect(screen.getByText("Broken Plugin")).toBeInTheDocument();
    expect(
      screen.getByText("Missing Contents/MacOS binary"),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Binary path unavailable"),
    ).not.toBeInTheDocument();
  });
});
