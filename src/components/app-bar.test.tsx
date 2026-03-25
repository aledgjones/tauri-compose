import { describe, it, expect, vi, beforeEach } from "vitest";
import { AppBar } from "./app-bar";
import {
  renderWithProviders,
  createAppState,
  screen,
} from "../services/test-utils";

describe("AppBar", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the drag region div", () => {
    const { container } = renderWithProviders(<AppBar title="Compose" />);
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
    expect(screen.getByTestId("app-bar-custom-title")).toHaveTextContent(
      "Compose",
    );
  });

  it("keeps the title row transparent to pointer input so dragging works", () => {
    const { container } = renderWithProviders(<AppBar title="Compose" />);

    expect(container.firstElementChild?.firstElementChild).toHaveClass(
      "pointer-events-none",
      "select-none",
    );
  });

  it("shows only the custom title when no file is open", () => {
    renderWithProviders(<AppBar title="Compose" />);

    expect(screen.getByTestId("app-bar-custom-title")).toHaveTextContent(
      "Compose",
    );
    expect(screen.queryByTestId("app-bar-filename")).toBeNull();
  });

  it("includes filename when current_file_path is set", () => {
    renderWithProviders(<AppBar title="Compose" showFilename />, {
      appState: createAppState({ current_file_path: "/home/user/song.sco" }),
    });

    expect(screen.getByTestId("app-bar-custom-title")).toHaveTextContent(
      "Compose",
    );
    expect(screen.getByTestId("app-bar-filename")).toHaveTextContent(
      "song.sco",
    );
  });

  it("does not render a filename pill when current_file_path is null", () => {
    renderWithProviders(<AppBar title="Compose" showFilename />);

    expect(screen.queryByTestId("app-bar-filename")).toBeNull();
  });

  it("does not render a filename in secondary windows even when a file is open", () => {
    renderWithProviders(<AppBar title="Debugger" />, {
      appState: createAppState({ current_file_path: "/home/user/song.sco" }),
    });

    expect(screen.queryByTestId("app-bar-filename")).toBeNull();
  });

  it("fades the custom title when the window is unfocused", () => {
    renderWithProviders(<AppBar title="Compose" />, {
      appState: createAppState({ is_window_focused: false }),
    });

    expect(screen.getByTestId("app-bar-custom-title").className).toContain(
      "text-neutral-100/35",
    );
  });
});
