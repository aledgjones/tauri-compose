import { describe, it, expect, vi } from "vitest";
import { screen } from "@testing-library/react";
import { renderWithProviders } from "../../services/test-utils";
import { Main } from "./index";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ show: vi.fn(), setTitle: vi.fn() }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((p: string) => p),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: vi.fn().mockImplementation(() => ({
    setFocus: vi.fn(),
    once: vi.fn(),
    close: vi.fn(),
  })),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
  emit: vi.fn().mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: vi.fn(),
  open: vi.fn(),
}));

vi.mock("sortablejs", () => ({
  default: { create: vi.fn().mockReturnValue({ destroy: vi.fn() }) },
}));

vi.mock("../../services/actions", () => ({
  actions: {
    ui: {
      setTab: vi.fn(),
      setLayout: vi.fn(),
      setFlow: vi.fn(),
      renderer: {
        setBounds: vi.fn(),
        toggleDebug: vi.fn(),
      },
    },
    score: {
      layout: { create: vi.fn(), select: vi.fn() },
      flow: {
        create: vi.fn(),
        select: vi.fn(),
        rename: vi.fn(),
        remove: vi.fn(),
        assignPlayer: vi.fn(),
        unassignPlayer: vi.fn(),
      },
      player: { create: vi.fn(), select: vi.fn() },
      instrument: { setSolo: vi.fn(), setMute: vi.fn(), setVolume: vi.fn() },
    },
    transport: {
      play: vi.fn(),
      stop: vi.fn(),
      rewind: vi.fn(),
    },
  },
}));

describe("Main Window", () => {
  it("renders the app bar drag region", () => {
    const { container } = renderWithProviders(<Main />);
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("renders all tab options", () => {
    renderWithProviders(<Main />);
    expect(screen.getByRole("button", { name: "Setup" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Write" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Engrave" })).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Sequence" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Print" })).toBeInTheDocument();
  });
});
