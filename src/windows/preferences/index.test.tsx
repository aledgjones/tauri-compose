import { describe, it, expect, vi } from "vitest";
import { screen } from "@testing-library/react";
import { Main } from "./index";
import { renderWithProviders } from "../../services/test-utils";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    show: vi.fn().mockResolvedValue(undefined),
    setTitle: vi.fn().mockResolvedValue(undefined),
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((p: string) => p),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: vi.fn().mockImplementation(() => ({
    once: vi.fn().mockResolvedValue(vi.fn()),
    close: vi.fn(),
    setFocus: vi.fn(),
  })),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn().mockResolvedValue(null),
  open: vi.fn().mockResolvedValue(null),
}));

vi.mock("../../services/actions", () => ({
  actions: {
    audio: {
      setDevice: vi.fn().mockResolvedValue(undefined),
      refreshDevices: vi.fn().mockResolvedValue(undefined),
      setDeviceSampleRate: vi.fn().mockResolvedValue(undefined),
    },
    preferences: {
      setAudition: vi.fn(),
    },
  },
}));

describe("Preferences Main", () => {
  it("renders without crashing", () => {
    const { container } = renderWithProviders(<Main />);
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("renders the Playback menu item", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("Playback")).toBeInTheDocument();
  });

  it("renders the Note Input and Editing menu item", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("Note Input and Editing")).toBeInTheDocument();
  });

  it("shows Device Setup content by default (Playback screen)", () => {
    renderWithProviders(<Main />);
    // 'Device Setup' appears as menu sub-item and as heading in content
    expect(
      screen.getByRole("heading", { name: "Device Setup" }),
    ).toBeInTheDocument();
  });
});
