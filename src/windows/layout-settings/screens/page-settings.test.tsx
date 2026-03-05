import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import { PageSettings } from "./page-settings";
import { renderWithProviders } from "../../../services/test-utils";
import { LayoutType } from "../../../services/types";

vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      engrave: {
        setSpace: vi.fn(),
        setPagePaddingTop: vi.fn(),
        setPagePaddingLeft: vi.fn(),
        setPagePaddingBottom: vi.fn(),
        setPagePaddingRight: vi.fn(),
        setFramePaddingTop: vi.fn(),
        setFramePaddingBottom: vi.fn(),
      },
    },
  },
}));

describe("PageSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the Space Size heading", () => {
    renderWithProviders(<PageSettings layoutType={LayoutType.FullScore} />);
    expect(screen.getByText("Space Size")).toBeInTheDocument();
  });

  it("renders the Page Margins heading", () => {
    renderWithProviders(<PageSettings layoutType={LayoutType.FullScore} />);
    expect(screen.getByText("Page Margins")).toBeInTheDocument();
  });

  it("renders the Music Frame Margins heading", () => {
    renderWithProviders(<PageSettings layoutType={LayoutType.FullScore} />);
    expect(screen.getByText("Music Frame Margins")).toBeInTheDocument();
  });

  it("renders the space size input", () => {
    renderWithProviders(<PageSettings layoutType={LayoutType.FullScore} />);
    expect(screen.getByText("Space size")).toBeInTheDocument();
  });

  it("renders all page margin inputs", () => {
    renderWithProviders(<PageSettings layoutType={LayoutType.FullScore} />);
    const tops = screen.getAllByText("Top");
    expect(tops.length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Left")).toBeInTheDocument();
    const bottoms = screen.getAllByText("Bottom");
    expect(bottoms.length).toBeGreaterThanOrEqual(1);
    expect(screen.getByText("Space Size")).toBeInTheDocument();
  });
});
