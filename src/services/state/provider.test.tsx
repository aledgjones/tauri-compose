import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { AppStateProvider } from "./provider";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn((p: string) => p),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: vi.fn().mockImplementation(() => ({
    setFocus: vi.fn(),
    once: vi.fn().mockResolvedValue(vi.fn()),
    close: vi.fn(),
  })),
}));

vi.mock("../actions", () => ({
  actions: {
    initialize: vi.fn().mockResolvedValue(undefined),
    transport: {
      play: vi.fn().mockResolvedValue(undefined),
      pause: vi.fn().mockResolvedValue(undefined),
      stop: vi.fn().mockResolvedValue(undefined),
      seek: vi.fn().mockResolvedValue(undefined),
      setTempo: vi.fn().mockResolvedValue(undefined),
      getState: vi.fn().mockResolvedValue({
        playing: false,
        current_tick: 0,
        tempo: 120,
      }),
    },
    file: {
      save: vi.fn().mockResolvedValue(undefined),
      saveAs: vi.fn().mockResolvedValue(undefined),
      load: vi.fn().mockResolvedValue(undefined),
      loadFromPath: vi.fn().mockResolvedValue(undefined),
    },
    reset: vi.fn().mockResolvedValue(undefined),
  },
}));

describe("AppStateProvider", () => {
  it("renders children", () => {
    render(
      <AppStateProvider>
        <div>child content</div>
      </AppStateProvider>,
    );
    expect(screen.getByText("child content")).toBeInTheDocument();
  });

  it("renders multiple children", () => {
    render(
      <AppStateProvider>
        <span>first</span>
        <span>second</span>
      </AppStateProvider>,
    );
    expect(screen.getByText("first")).toBeInTheDocument();
    expect(screen.getByText("second")).toBeInTheDocument();
  });

  it("renders with isMainWindow=false (default)", () => {
    render(
      <AppStateProvider isMainWindow={false}>
        <div>non-main</div>
      </AppStateProvider>,
    );
    expect(screen.getByText("non-main")).toBeInTheDocument();
  });

  it("renders with isMainWindow=true", () => {
    render(
      <AppStateProvider isMainWindow={true}>
        <div>main-window-child</div>
      </AppStateProvider>,
    );
    expect(screen.getByText("main-window-child")).toBeInTheDocument();
  });
});
