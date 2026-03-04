import { describe, it, expect, vi, beforeEach } from "vitest";
import { AppBar } from "./app-bar";
import { renderWithProviders, createAppState } from "../services/test-utils";

// Mock Tauri window API
const setTitle = vi.fn();
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ setTitle }),
}));

describe("AppBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the drag region div", () => {
    const { container } = renderWithProviders(<AppBar title="Compose" />);
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("calls setTitle with formatted title on mount", async () => {
    renderWithProviders(<AppBar title="Compose" />);
    // Let the async effect run
    await vi.runAllTimersAsync().catch(() => {});
    expect(setTitle).toHaveBeenCalledWith(expect.stringContaining("Compose"));
  });

  it("includes filename in title when current_file_path is set", async () => {
    renderWithProviders(<AppBar title="Compose" />, {
      appState: createAppState({ current_file_path: "/home/user/song.sco" }),
    });
    await vi.runAllTimersAsync().catch(() => {});
    expect(setTitle).toHaveBeenCalledWith(expect.stringContaining("song.sco"));
  });

  it("uses 'Untitled' when current_file_path is null", async () => {
    renderWithProviders(<AppBar title="Compose" />);
    await vi.runAllTimersAsync().catch(() => {});
    expect(setTitle).toHaveBeenCalledWith(expect.stringContaining("Untitled"));
  });
});
