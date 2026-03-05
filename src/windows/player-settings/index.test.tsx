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
    score: {
      config: {
        autoCountStyles: {
          setSolo: vi.fn(),
          setSection: vi.fn(),
        },
      },
    },
  },
}));

describe("Player Settings Main", () => {
  it("renders without crashing", () => {
    const { container } = renderWithProviders(<Main />);
    // AppBar renders a drag region div (title is set via Tauri window.setTitle, not DOM text)
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("renders the General menu item", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("General")).toBeInTheDocument();
  });

  it("renders the General content by default", () => {
    renderWithProviders(<Main />);
    expect(
      screen.getByRole("heading", { name: "Systemic Barlines" }),
    ).toBeInTheDocument();
  });
});
