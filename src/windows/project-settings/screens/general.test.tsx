import { describe, it, expect, vi } from "vitest";
import { screen } from "@testing-library/react";
import { General } from "./general";
import { renderWithProviders } from "../../../services/test-utils";

vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      meta: {
        setTitle: vi.fn(),
        setSubtitle: vi.fn(),
        setDedication: vi.fn(),
        setComposer: vi.fn(),
        setArranger: vi.fn(),
        setLyricist: vi.fn(),
        setArtist: vi.fn(),
        setCopyist: vi.fn(),
        setPublisher: vi.fn(),
        setEditor: vi.fn(),
        setCopyright: vi.fn(),
        setWorkNumber: vi.fn(),
        setCompositionYear: vi.fn(),
        setComposerDates: vi.fn(),
        setOtherInformation: vi.fn(),
      },
    },
  },
}));

describe("General (project settings)", () => {
  it("renders without crashing", () => {
    renderWithProviders(<General />);
    expect(screen.getByText("Project Information")).toBeInTheDocument();
  });

  it("renders Title input", () => {
    renderWithProviders(<General />);
    expect(screen.getByText("Title")).toBeInTheDocument();
  });

  it("renders Composer input", () => {
    renderWithProviders(<General />);
    expect(screen.getByText("Composer")).toBeInTheDocument();
  });

  it("renders Copyright textarea", () => {
    renderWithProviders(<General />);
    expect(screen.getByText("Copyright")).toBeInTheDocument();
  });
});
