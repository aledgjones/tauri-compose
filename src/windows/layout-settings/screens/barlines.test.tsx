import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { Barlines } from "./barlines";
import { renderWithProviders } from "../../../services/test-utils";
import { LayoutType } from "../../../services/types";

vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      engrave: {
        setSystemicBarlineSingleInstrumentSystem: vi.fn(),
      },
    },
  },
}));

import { actions } from "../../../services/actions";

describe("Barlines", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the Systemic Barlines heading", () => {
    renderWithProviders(<Barlines layoutType={LayoutType.FullScore} />);
    expect(screen.getByText("Systemic Barlines")).toBeInTheDocument();
  });

  it("renders the switch label", () => {
    renderWithProviders(<Barlines layoutType={LayoutType.FullScore} />);
    expect(
      screen.getByText("Show systemic barlines for single staves"),
    ).toBeInTheDocument();
  });

  it("shows 'Show systemic barlines for systems with only one staff' when value is true (FullScore default)", () => {
    // initialAppState has systemic_barline_single_instrument_system: true for FullScore
    renderWithProviders(<Barlines layoutType={LayoutType.FullScore} />);
    expect(
      screen.getByText(
        "Show systemic barlines for systems with only one staff.",
      ),
    ).toBeInTheDocument();
  });

  it("shows the alternate description when value is false (Part default)", () => {
    // initialAppState has systemic_barline_single_instrument_system: false for Part
    renderWithProviders(<Barlines layoutType={LayoutType.Part} />);
    expect(
      screen.getByText(
        "Show systemic barlines for systems with two or more staves.",
      ),
    ).toBeInTheDocument();
  });

  it("calls setSystemicBarlineSingleInstrumentSystem when switch is toggled", () => {
    renderWithProviders(<Barlines layoutType={LayoutType.FullScore} />);
    fireEvent.click(screen.getByRole("switch"));
    expect(
      actions.score.engrave.setSystemicBarlineSingleInstrumentSystem,
    ).toHaveBeenCalledWith(LayoutType.FullScore, expect.any(Boolean));
  });
});
