import { describe, it, expect, vi } from "vitest";
import { screen } from "@testing-library/react";
import { General } from "./general";
import { renderWithProviders } from "../../../services/test-utils";

vi.mock("../../../services/actions", () => ({
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

describe("General (player settings)", () => {
  it("renders without crashing", () => {
    renderWithProviders(<General />);
    expect(screen.getByText("Systemic Barlines")).toBeInTheDocument();
  });

  it("renders solo numbering style select", () => {
    renderWithProviders(<General />);
    expect(
      screen.getByText("Numbering style for single players"),
    ).toBeInTheDocument();
  });

  it("renders section numbering style select", () => {
    renderWithProviders(<General />);
    expect(
      screen.getByText("Numbering style for section players"),
    ).toBeInTheDocument();
  });
});
