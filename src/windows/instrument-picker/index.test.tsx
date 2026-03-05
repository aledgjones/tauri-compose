import { describe, it, expect, vi } from "vitest";
import { screen } from "@testing-library/react";
import { renderWithProviders } from "../../services/test-utils";
import { Main } from "./index";

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    show: vi.fn(),
    center: vi.fn(),
    setTitle: vi.fn(),
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

vi.mock("@tauri-apps/api/event", () => ({
  emit: vi.fn().mockResolvedValue(undefined),
}));

describe("InstrumentPicker Main", () => {
  it("renders the app bar drag region", () => {
    const { container } = renderWithProviders(<Main />);
    expect(
      container.querySelector("[data-tauri-drag-region]"),
    ).toBeInTheDocument();
  });

  it("renders the Select Instrument button", () => {
    renderWithProviders(<Main />);
    expect(
      screen.getByRole("button", { name: "Select Instrument" }),
    ).toBeInTheDocument();
  });
});
