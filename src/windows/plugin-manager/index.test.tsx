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
  editor_host_validation: null,
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
    expect(
      screen.getByRole("button", {
        name: "Open native editor host for Grand Piano",
      }),
    ).toBeInTheDocument();
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

  it("keeps editor-blocked plugins when Available only is enabled", () => {
    renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [
          makePlugin(),
          makePlugin({
            id: "vst3:/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            name: "HALion Sonic",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            binary_path:
              "/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3/Contents/MacOS/HALion Sonic",
            editor_host_validation: {
              status: "blocked",
              phase: "policy_gate",
              message:
                "Blocked for the bounded native editor host by recorded compatibility evidence.",
              capability_evidence: null,
            },
          }),
        ],
      },
    });

    fireEvent.click(screen.getByRole("switch"));

    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
    expect(screen.getByText("HALion Sonic")).toBeInTheDocument();
  });

  it("shows the plugin path for unavailable plugins", () => {
    renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [
          makePlugin({
            id: "vst3:/Library/Audio/Plug-Ins/VST3/Broken.vst3",
            name: "Broken Plugin",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/Broken.vst3",
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
      screen.getByText("/Library/Audio/Plug-Ins/VST3/Broken.vst3"),
    ).toBeInTheDocument();
  });

  it("shows bounded native editor-host blocked state for excluded plugins", () => {
    renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [
          makePlugin({
            editor_host_validation: {
              status: "blocked",
              phase: "policy_gate",
              message:
                "Blocked for the bounded native editor host by recorded compatibility evidence.",
              capability_evidence: null,
            },
          }),
        ],
      },
    });

    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
    expect(screen.getByLabelText("Status: Available")).toBeInTheDocument();
    expect(
      screen.getByText(/Contents\/MacOS\/Grand Piano/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Native editor status:\s*Blocked/i),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", {
        name: "Open native editor host for Grand Piano",
      }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Maintainer preview")).toBeInTheDocument();
  });

  it("orders available plugins before unsupported and unavailable entries", () => {
    const { container } = renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [
          makePlugin({
            id: "unsupported-zebra",
            name: "Zebra Synth",
            editor_host_validation: {
              status: "blocked",
              phase: "policy_gate",
              message:
                "Blocked for the bounded native editor host by recorded compatibility evidence.",
              capability_evidence: null,
            },
          }),
          makePlugin({
            id: "available-strings",
            name: "Amber Strings",
          }),
          makePlugin({
            id: "unavailable-broken",
            name: "Broken Plugin",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/Broken.vst3",
            binary_path: null,
            availability: "unavailable",
            validation_failure: {
              kind: "missing_bundle_binary",
              message: "Missing Contents/MacOS binary",
            },
          }),
          makePlugin({
            id: "available-piano",
            name: "Grand Piano",
          }),
        ],
      },
    });

    const renderedNames = Array.from(
      container.querySelectorAll(".text-white.text-sm.truncate.font-semibold"),
    ).map((element) => element.textContent);

    expect(renderedNames).toEqual([
      "Amber Strings",
      "Grand Piano",
      "Zebra Synth",
      "Broken Plugin",
    ]);
  });
});
