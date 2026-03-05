import { describe, it, expect, vi } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
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
      engrave: {
        setSystemicBarlineSingleInstrumentSystem: vi.fn(),
        setBracketingApproach: vi.fn(),
        setBracketStyle: vi.fn(),
        setBracketSingleStaves: vi.fn(),
        setBaseNoteSpace: vi.fn(),
        setMinimumNoteSpace: vi.fn(),
        setNoteSpaceRatio: vi.fn(),
        setSpace: vi.fn(),
        setPagePadding: vi.fn(),
        setFramePadding: vi.fn(),
      },
    },
  },
}));

describe("Layout Settings Main", () => {
  it("renders without crashing", () => {
    const { container } = renderWithProviders(<Main />);
    // AppBar renders a drag region div (title set via Tauri window.setTitle, not DOM text)
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("renders menu items — Barlines", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("Barlines")).toBeInTheDocument();
  });

  it("renders menu items — Brackets and Braces", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("Brackets and Braces")).toBeInTheDocument();
  });

  it("renders menu items — Note Spacing", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("Note Spacing")).toBeInTheDocument();
  });

  it("renders menu items — Page Settings", () => {
    renderWithProviders(<Main />);
    expect(screen.getByText("Page Settings")).toBeInTheDocument();
  });

  it("shows Barlines content by default", () => {
    renderWithProviders(<Main />);
    // 'Systemic Barlines' appears as both a menu sub-item and an h3 heading
    expect(
      screen.getByRole("heading", { name: "Systemic Barlines" }),
    ).toBeInTheDocument();
  });

  it("switches to Note Spacing content when menu item clicked", () => {
    renderWithProviders(<Main />);
    fireEvent.click(screen.getByText("Note Spacing"));
    expect(screen.getByText("Base Note Space")).toBeInTheDocument();
  });
});
