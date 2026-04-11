import { beforeEach, describe, it, expect, vi } from "vitest";
import { screen, userEvent, waitFor } from "../../services/test-utils";
import { renderWithProviders } from "../../services/test-utils";
import { Main } from "./index";

const {
  mockToggleDebug,
  mockPrepareProbeLog,
  mockProbeLoadInit,
  mockReadProbeLog,
} = vi.hoisted(() => ({
  mockToggleDebug: vi.fn(),
  mockPrepareProbeLog: vi.fn(),
  mockProbeLoadInit: vi.fn(),
  mockReadProbeLog: vi.fn(),
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
    pluginInventory: {
      prepareProbeLog: mockPrepareProbeLog,
      probeLoadInit: mockProbeLoadInit,
      readProbeLog: mockReadProbeLog,
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
    mockPrepareProbeLog.mockReset();
    mockProbeLoadInit.mockReset();
    mockReadProbeLog.mockReset();
    mockReadProbeLog.mockResolvedValue({
      path: "/tmp/plugin-editor-poc-load-init-probe.log",
      profile: null,
      crash_hint: null,
      contents: null,
    });
  });

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

  it("loads the persisted probe log on mount", async () => {
    mockReadProbeLog.mockResolvedValueOnce({
      path: "/tmp/plugin-editor-poc-load-init-probe.log",
      profile: "controller_init_host_only_lifecycle_aligned",
      crash_hint: {
        phase: "controller_initialize",
        last_line:
          "[1] phase=controller_initialize status=starting message=About to initialize the VST3 edit controller.",
        message:
          "Likely hard crash during controller_initialize. The last recorded probe event started but never completed or failed.",
      },
      contents:
        "[1] phase=controller_initialize status=starting message=About to initialize the VST3 edit controller.",
    });

    renderWithProviders(<Main />);

    await waitFor(() => {
      expect(mockReadProbeLog).toHaveBeenCalled();
    });

    expect(
      screen.getByText(
        /Log Path: \/tmp\/plugin-editor-poc-load-init-probe\.log/i,
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Likely hard crash during controller_initialize/i),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        /Probe Profile: Controller Init Host-Only Lifecycle Aligned/i,
      ),
    ).toBeInTheDocument();
    expect(
      screen.getAllByText(/phase=controller_initialize status=starting/i),
    ).toHaveLength(2);
  });

  it("runs the maintainer load/init probe for the selected inventory entry", async () => {
    mockPrepareProbeLog.mockResolvedValue({
      path: "/tmp/plugin-editor-poc-load-init-probe.log",
      profile: "default",
      crash_hint: null,
      contents:
        "[0] phase=general status=info message=Prepared the maintainer load/init probe log before invoking the risky host probe.",
    });
    mockProbeLoadInit.mockResolvedValue({
      status: "blocked",
      phase: "policy_gate",
      message: "Maintainer probe captured plugin-side failure.",
      capability_evidence: null,
      maintainer_load_init_probe: {
        status: "blocked_by_plugin_failure",
        last_successful_phase: "controller_create",
        first_failing_phase: "controller_initialize",
        first_failing_message: "HALion blocked at controller initialize.",
        summary: "HALion blocked at controller initialize.",
        phase_results: [],
      },
    });
    mockReadProbeLog
      .mockResolvedValueOnce({
        path: "/tmp/plugin-editor-poc-load-init-probe.log",
        profile: null,
        crash_hint: null,
        contents: null,
      })
      .mockResolvedValueOnce({
        path: "/tmp/plugin-editor-poc-load-init-probe.log",
        profile: "default",
        crash_hint: null,
        contents:
          "[2] phase=controller_initialize status=failed message=HALion blocked at controller initialize.",
      });

    const user = userEvent.setup();

    renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [
          {
            id: "vst3:halion-sonic",
            name: "HALion Sonic",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            binary_path: null,
            source_root: "/Library/Audio/Plug-Ins/VST3",
            availability: "available",
            validation_failure: null,
            editor_host_validation: null,
          },
        ],
      },
    });

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Run Load/Init Probe" }),
      ).not.toBeDisabled();
    });

    await user.click(
      screen.getByRole("button", { name: "Run Load/Init Probe" }),
    );

    await waitFor(() => {
      expect(mockPrepareProbeLog).toHaveBeenCalledWith({
        plugin_id: "vst3:halion-sonic",
        plugin_path: null,
        profile: "default",
      });
      expect(mockProbeLoadInit).toHaveBeenCalledWith({
        plugin_id: "vst3:halion-sonic",
        plugin_path: null,
        profile: "default",
      });
    });

    expect(
      screen.getByText("HALion blocked at controller initialize."),
    ).toBeInTheDocument();

    await waitFor(() => {
      expect(
        screen.getByText(/phase=controller_initialize status=failed/i),
      ).toBeInTheDocument();
    });
  });

  it("reuses the persisted host-only lifecycle-aligned profile for the next probe run", async () => {
    mockReadProbeLog.mockResolvedValueOnce({
      path: "/tmp/plugin-editor-poc-load-init-probe.log",
      profile: "controller_init_host_only_lifecycle_aligned",
      crash_hint: null,
      contents: "probe_profile=controller_init_host_only_lifecycle_aligned",
    });
    mockPrepareProbeLog.mockResolvedValue({
      path: "/tmp/plugin-editor-poc-load-init-probe.log",
      profile: "controller_init_host_only_lifecycle_aligned",
      crash_hint: null,
      contents: "probe_profile=controller_init_host_only_lifecycle_aligned",
    });
    mockProbeLoadInit.mockResolvedValue({
      status: "blocked",
      phase: "policy_gate",
      message: "Maintainer probe captured plugin-side failure.",
      capability_evidence: null,
      maintainer_load_init_probe: {
        status: "blocked_by_plugin_failure",
        last_successful_phase: "controller_create",
        first_failing_phase: "controller_initialize",
        first_failing_message: "HALion blocked at controller initialize.",
        summary:
          "Maintainer load/init probe (controller_init_host_only_lifecycle_aligned) first failed at controller_initialize and appears plugin-side: HALion blocked at controller initialize.",
        phase_results: [],
      },
    });

    const user = userEvent.setup();

    renderWithProviders(<Main />, {
      pluginInventoryState: {
        entries: [
          {
            id: "vst3:halion-sonic",
            name: "HALion Sonic",
            bundle_path: "/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
            binary_path: null,
            source_root: "/Library/Audio/Plug-Ins/VST3",
            availability: "available",
            validation_failure: null,
            editor_host_validation: null,
          },
        ],
      },
    });

    await waitFor(() => {
      expect(
        screen.getByText(
          /Probe Profile: Controller Init Host-Only Lifecycle Aligned/i,
        ),
      ).toBeInTheDocument();
    });

    await user.click(
      screen.getByRole("button", { name: "Run Load/Init Probe" }),
    );

    await waitFor(() => {
      expect(mockPrepareProbeLog).toHaveBeenCalledWith({
        plugin_id: "vst3:halion-sonic",
        plugin_path: null,
        profile: "controller_init_host_only_lifecycle_aligned",
      });
      expect(mockProbeLoadInit).toHaveBeenCalledWith({
        plugin_id: "vst3:halion-sonic",
        plugin_path: null,
        profile: "controller_init_host_only_lifecycle_aligned",
      });
    });
  });

  it("prefers a manual plugin path override when running the probe", async () => {
    mockPrepareProbeLog.mockResolvedValue({
      path: "/tmp/plugin-editor-poc-load-init-probe.log",
      profile: "default",
      crash_hint: null,
      contents:
        "[0] phase=general status=info message=Prepared the maintainer load/init probe log before invoking the risky host probe.",
    });
    mockProbeLoadInit.mockRejectedValue(new Error("plugin probe crashed"));

    const user = userEvent.setup();

    renderWithProviders(<Main />);

    await user.type(
      screen.getByLabelText("Plugin path override"),
      "/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
    );

    await user.click(
      screen.getByRole("button", { name: "Run Load/Init Probe" }),
    );

    await waitFor(() => {
      expect(mockPrepareProbeLog).toHaveBeenCalledWith({
        plugin_id: null,
        plugin_path: "/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
        profile: "default",
      });
      expect(mockProbeLoadInit).toHaveBeenCalledWith({
        plugin_id: null,
        plugin_path: "/Library/Audio/Plug-Ins/VST3/HALion Sonic.vst3",
        profile: "default",
      });
    });

    expect(screen.getByRole("alert")).toHaveTextContent("plugin probe crashed");
  });
});
