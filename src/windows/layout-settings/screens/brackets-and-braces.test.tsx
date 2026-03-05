import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import { BracketsAndBraces } from "./brackets-and-braces";
import {
  renderWithProviders,
  createAppState,
} from "../../../services/test-utils";
import { LayoutType } from "../../../services/types";

vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      engrave: {
        setBracketingApproach: vi.fn(),
        setBracketStyle: vi.fn(),
        setBracketSingleStaves: vi.fn(),
      },
    },
  },
}));

describe("BracketsAndBraces", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the Brackets heading", () => {
    renderWithProviders(
      <BracketsAndBraces layoutType={LayoutType.FullScore} />,
    );
    expect(screen.getByText("Brackets")).toBeInTheDocument();
  });

  it("renders the Bracketed Groups heading", () => {
    renderWithProviders(
      <BracketsAndBraces layoutType={LayoutType.FullScore} />,
    );
    expect(screen.getByText("Bracketed Groups")).toBeInTheDocument();
  });

  it("renders the Bracketing Approach select", () => {
    renderWithProviders(
      <BracketsAndBraces layoutType={LayoutType.FullScore} />,
    );
    expect(screen.getByText("Bracketing Approach")).toBeInTheDocument();
  });

  it("renders the Bracketing Style select", () => {
    renderWithProviders(
      <BracketsAndBraces layoutType={LayoutType.FullScore} />,
    );
    expect(screen.getByText("Bracketing Style")).toBeInTheDocument();
  });

  it("renders the 'Show brackets for single staves' switch", () => {
    renderWithProviders(
      <BracketsAndBraces layoutType={LayoutType.FullScore} />,
    );
    expect(
      screen.getByText("Show brackets for single staves"),
    ).toBeInTheDocument();
  });

  it("renders description for bracket_single_staves=false (FullScore default)", () => {
    // initialAppState has bracket_single_staves: false for FullScore
    renderWithProviders(
      <BracketsAndBraces layoutType={LayoutType.FullScore} />,
    );
    expect(
      screen.getByText("Show brackets for systems with two or more staves."),
    ).toBeInTheDocument();
  });

  it("renders without crashing for Part layout", () => {
    renderWithProviders(<BracketsAndBraces layoutType={LayoutType.Part} />);
    expect(screen.getByText("Brackets")).toBeInTheDocument();
  });

  it("renders alternate description when bracket_single_staves=true", () => {
    const appState = createAppState({
      score: {
        engraves: {
          by_layout_type: {
            [LayoutType.FullScore]: {
              bracketing_approach: 0,
              bracket_style: 0,
              bracket_single_staves: true,
            },
          },
        },
      } as never,
    });
    renderWithProviders(
      <BracketsAndBraces layoutType={LayoutType.FullScore} />,
      { appState },
    );
    expect(
      screen.getByText("Show brackets for systems with only one staff."),
    ).toBeInTheDocument();
  });
});
