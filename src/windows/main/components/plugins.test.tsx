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
    sampler: {
      rescanPlugins: vi.fn().mockResolvedValue(undefined),
      audition: vi.fn(),
    },
  },
}));

import { actions } from "../../../services/actions";

const makePlugin = (overrides = {}) => ({
  id: "com.vendor.piano",
  name: "Grand Piano",
  path: "/plugins/grand-piano",
  author: "VendorCo",
  version: "1.0.0",
  articulations: [],
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

  it("renders Active only label", () => {
    renderWithProviders(<PluginsToolbar {...defaults} />);
    expect(screen.getByText("Active only")).toBeInTheDocument();
  });

  it("calls rescanPlugins when refresh button is clicked", async () => {
    renderWithProviders(<PluginsToolbar {...defaults} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[0]);
    await waitFor(() => {
      expect(actions.sampler.rescanPlugins).toHaveBeenCalled();
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
});

describe("Plugins", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders 'No plugins found' when sampler has no plugins", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />);
    expect(screen.getByText("No plugins found")).toBeInTheDocument();
  });

  it("renders plugin names when plugins exist", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      samplerState: {
        plugins: [makePlugin()],
        loaded_plugin_ids: [],
      },
    });
    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
  });

  it("shows 'No plugins match' when search term filters all plugins", () => {
    renderWithProviders(
      <Plugins searchTerm="xyznothing" showActiveOnly={false} />,
      {
        samplerState: {
          plugins: [makePlugin()],
          loaded_plugin_ids: [],
        },
      },
    );
    expect(
      screen.getByText("No plugins match your filters"),
    ).toBeInTheDocument();
  });

  it("shows 'No plugins match' when showActiveOnly=true and none loaded", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={true} />, {
      samplerState: {
        plugins: [makePlugin({ id: "com.vendor.piano" })],
        loaded_plugin_ids: [],
      },
    });
    expect(
      screen.getByText("No plugins match your filters"),
    ).toBeInTheDocument();
  });

  it("does not render a plugin-settings entry point on plugin tiles", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      samplerState: {
        plugins: [makePlugin()],
        loaded_plugin_ids: ["com.vendor.piano"],
      },
    });

    expect(
      screen.queryByRole("button", {
        name: /plugin settings|open settings|open plugin/i,
      }),
    ).not.toBeInTheDocument();
  });

  it("shows plugin when showActiveOnly=true and plugin is loaded", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={true} />, {
      samplerState: {
        plugins: [makePlugin({ id: "com.vendor.piano" })],
        loaded_plugin_ids: ["com.vendor.piano"],
      },
    });
    expect(screen.getByText("Grand Piano")).toBeInTheDocument();
  });

  it("renders plugin version", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      samplerState: {
        plugins: [makePlugin({ version: "2.3.1" })],
        loaded_plugin_ids: [],
      },
    });
    expect(screen.getByText("v2.3.1")).toBeInTheDocument();
  });

  it("renders plugin author", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      samplerState: {
        plugins: [makePlugin({ author: "AcmeSounds" })],
        loaded_plugin_ids: [],
      },
    });
    expect(screen.getByText("by AcmeSounds")).toBeInTheDocument();
  });

  it("renders multiple plugins", () => {
    renderWithProviders(<Plugins searchTerm="" showActiveOnly={false} />, {
      samplerState: {
        plugins: [
          makePlugin({ id: "a", name: "Alpha Synth" }),
          makePlugin({ id: "b", name: "Beta Bass" }),
        ],
        loaded_plugin_ids: [],
      },
    });
    expect(screen.getByText("Alpha Synth")).toBeInTheDocument();
    expect(screen.getByText("Beta Bass")).toBeInTheDocument();
  });
});
