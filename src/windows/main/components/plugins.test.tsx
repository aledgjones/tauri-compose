import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent, waitFor } from "@testing-library/react";
import { renderWithProviders } from "../../../services/test-utils";
import { PluginsToolbar, Plugins } from "./plugins";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((p: string) => `asset://${p}`),
}));

vi.mock("../../../services/actions", () => ({
  actions: {
    pluginInventory: {
      discover: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

import { actions } from "../../../services/actions";

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

describe("PluginsToolbar", () => {
  const defaults = {
    searchTerm: "",
    onSearchTermChange: vi.fn(),
    showActiveOnly: false,
    onShowActiveOnlyChange: vi.fn(),
  };

  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing", () => {
    const { container } = renderWithProviders(<PluginsToolbar {...defaults} />);
    expect(container).toBeInTheDocument();
  });

  it("renders the refresh button", () => {
    renderWithProviders(<PluginsToolbar {...defaults} />);
    expect(screen.getAllByRole("button").length).toBeGreaterThanOrEqual(1);
  });

  it("renders Available only label", () => {
    renderWithProviders(<PluginsToolbar {...defaults} />);
    expect(screen.getByText("Available only")).toBeInTheDocument();
  });

  it("can hide the refresh button for browse-only windows", () => {
    renderWithProviders(
      <PluginsToolbar {...defaults} showRefreshButton={false} />,
    );

    expect(
      screen.queryByRole("button", { name: /refresh plugin inventory/i }),
    ).not.toBeInTheDocument();
  });

  it("calls discover when refresh button is clicked", async () => {
    renderWithProviders(<PluginsToolbar {...defaults} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[0]);
    await waitFor(() => {
      expect(actions.pluginInventory.discover).toHaveBeenCalled();
    });
  });

  it("calls onShowActiveOnlyChange when Active only switch is toggled", () => {
    const onShowActiveOnlyChange = vi.fn();
    renderWithProviders(
      <PluginsToolbar
        {...defaults}
        onShowActiveOnlyChange={onShowActiveOnlyChange}
      />,
    );
    // Switch role
    const switchEl = screen.getByRole("switch");
    fireEvent.click(switchEl);
    expect(onShowActiveOnlyChange).toHaveBeenCalledWith(true);
  });

  it("renders search input", () => {
    renderWithProviders(<PluginsToolbar {...defaults} />);
    expect(
      screen.getByPlaceholderText("Search plugins..."),
    ).toBeInTheDocument();
  });

  it("lets the search input expand to the remaining toolbar width", () => {
    renderWithProviders(<PluginsToolbar {...defaults} />);

    expect(
      screen.getByPlaceholderText("Search plugins...").closest("div")
        ?.parentElement?.className,
    ).toContain("flex-1");
  });
});

describe("Plugins", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders 'No plugins discovered yet' when inventory is empty", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />);
    expect(screen.getByText("No plugins discovered yet")).toBeInTheDocument();
  });

  it("renders plugin names when plugins exist", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [makePlugin()],
      },
    });
    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
    expect(screen.getByText("Available")).toBeInTheDocument();
    expect(screen.getByLabelText("Status: Available")).toBeInTheDocument();
    expect(screen.getByLabelText("Status: Available").className).toContain(
      "bg-emerald-500/10",
    );
  });

  it("shows 'No plugins match' when search term filters all plugins", () => {
    renderWithProviders(
      <Plugins searchTerm="xyznothing" showActiveOnly={false} />,
      {
        pluginInventoryState: {
          entries: [makePlugin()],
        },
      },
    );
    expect(
      screen.getByText("No plugins match your filters"),
    ).toBeInTheDocument();
  });

  it("shows 'No plugins match' when showActiveOnly=true and none are available", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={true} />, {
      pluginInventoryState: {
        entries: [
          makePlugin({
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
    expect(
      screen.getByText("No plugins match your filters"),
    ).toBeInTheDocument();
  });

  it("does not render a plugin-settings entry point on plugin tiles", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [makePlugin()],
      },
    });

    expect(
      screen.queryByRole("button", {
        name: /plugin settings|open settings|open plugin/i,
      }),
    ).not.toBeInTheDocument();
  });

  it("shows plugin when showActiveOnly=true and plugin is available", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={true} />, {
      pluginInventoryState: {
        entries: [makePlugin()],
      },
    });
    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
  });

  it("renders plugin binary path", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [makePlugin()],
      },
    });
    expect(
      screen.getByText(/Contents\/MacOS\/Grand Piano/),
    ).toBeInTheDocument();
  });

  it("renders the shared compact plugin row", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [makePlugin()],
      },
    });

    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
    expect(
      screen.getByText(/Contents\/MacOS\/Grand Piano/),
    ).toBeInTheDocument();
    expect(
      screen.getByTestId(
        "plugin-badge-vst3:/Library/Audio/Plug-Ins/VST3/Grand Piano.vst3",
      ),
    ).toHaveTextContent("GP");
    expect(screen.queryByText("Root")).not.toBeInTheDocument();
    expect(screen.queryByText("Binary")).not.toBeInTheDocument();
    expect(screen.getByTestId("plugins-list-filler").className).toContain(
      "bg-neutral-900",
    );
  });

  it("shows a compact validation message for unavailable plugins", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [
          makePlugin({
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

    expect(
      screen.getByText("Missing Contents/MacOS binary"),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("Binary path unavailable"),
    ).not.toBeInTheDocument();
  });

  it("uses a square plugin-name badge", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [makePlugin({ id: "a", name: "Alpha" })],
      },
    });

    const badge = screen.getByTestId("plugin-badge-a");

    expect(badge).toHaveTextContent("AL");
    expect(badge.className).toContain("h-10");
    expect(badge.className).toContain("w-10");
  });

  it("ignores punctuation when deriving badge initials", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [
          makePlugin({ id: "a", name: "Originals - Piano" }),
          makePlugin({ id: "b", name: "Originals Piano" }),
          makePlugin({ id: "c", name: "OriginalsPiano" }),
          makePlugin({ id: "d", name: "Originals ! -" }),
          makePlugin({ id: "e", name: " ! - " }),
        ],
      },
    });

    expect(screen.getByTestId("plugin-badge-a")).toHaveTextContent("OP");
    expect(screen.getByTestId("plugin-badge-b")).toHaveTextContent("OP");
    expect(screen.getByTestId("plugin-badge-c")).toHaveTextContent("OR");
    expect(screen.getByTestId("plugin-badge-d")).toHaveTextContent("OR");
    expect(screen.getByTestId("plugin-badge-e")).toHaveTextContent("?");
  });

  it("renders plugin issue details for problematic bundles", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [
          makePlugin({
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
    expect(screen.getByText("Unavailable")).toBeInTheDocument();
    expect(screen.getByLabelText("Status: Unavailable")).toBeInTheDocument();
    expect(screen.getByLabelText("Status: Unavailable").className).toContain(
      "bg-amber-500/10",
    );
    expect(
      screen.getByText("Missing Contents/MacOS binary"),
    ).toBeInTheDocument();
  });

  it("renders multiple plugins", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      pluginInventoryState: {
        entries: [
          makePlugin({ id: "a", name: "Alpha Synth" }),
          makePlugin({ id: "b", name: "Beta Bass" }),
        ],
      },
    });
    expect(screen.getByText("Alpha Synth")).toBeInTheDocument();
    expect(screen.getByText("Beta Bass")).toBeInTheDocument();
  });
});
