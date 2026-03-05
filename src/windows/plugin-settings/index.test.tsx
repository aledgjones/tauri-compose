import { describe, it, expect, vi } from "vitest";
import { screen } from "@testing-library/react";
import { renderWithProviders } from "../../services/test-utils";
import { Main } from "./index";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: vi.fn((p: string) => p),
}));

vi.mock("../../services/actions", () => ({
  actions: {
    sampler: {
      audition: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

describe("PluginSettings Main", () => {
  it("renders 'Plugin not found' when no plugin matches the URL param", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("Plugin not found")).toBeInTheDocument();
  });

  it("renders the keyboard when a plugin is present in sampler state", () => {
    const articulation = {
      id: "art-1",
      name: "Sustain",
      range: [[21, 108]] as Array<[number, number]>,
    };
    const plugin = {
      id: "plugin-1",
      name: "Grand Piano",
      path: "/instruments/piano",
      author: "Acme",
      version: "1.0",
      cover_image: undefined,
      articulations: [articulation],
    };

    // Simulate URL having ?id=plugin-1
    Object.defineProperty(window, "location", {
      value: { search: "?id=plugin-1" },
      writable: true,
    });

    const { container } = renderWithProviders(<Main />, {
      samplerState: {
        plugins: [plugin],
      },
    });

    // Plugin found: renders keyboard (white keys are buttons)
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBeGreaterThan(0);

    // Reset location
    Object.defineProperty(window, "location", {
      value: { search: "" },
      writable: true,
    });
  });
});
