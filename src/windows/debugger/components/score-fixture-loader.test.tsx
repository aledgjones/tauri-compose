import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ScoreFixtureLoader } from "./score-fixture-loader";

const { mockLoadScoreFixture } = vi.hoisted(() => ({
  mockLoadScoreFixture: vi.fn(),
}));

vi.mock("../../../services/actions", () => ({
  actions: {
    debug: {
      loadScoreFixture: mockLoadScoreFixture,
    },
  },
}));

describe("ScoreFixtureLoader", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockLoadScoreFixture.mockResolvedValue(undefined);
  });

  it("keeps included cases collapsed by default", () => {
    render(<ScoreFixtureLoader />);

    expect(
      screen.getByRole("button", { name: /Included cases/i }),
    ).toHaveAttribute("aria-expanded", "false");
    expect(
      screen.queryByText(/Four straight quarter notes/i),
    ).not.toBeInTheDocument();
  });

  it("expands included cases when toggled", async () => {
    const user = userEvent.setup();
    render(<ScoreFixtureLoader />);

    const toggle = screen.getByRole("button", { name: /Included cases/i });
    await user.click(toggle);

    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(
      screen.getByText(/Four straight quarter notes/i),
    ).toBeInTheDocument();
  });

  it("loads the selected fixture and shows success status", async () => {
    const user = userEvent.setup();
    render(<ScoreFixtureLoader />);

    await user.click(screen.getByRole("button", { name: /^Load$/i }));

    expect(mockLoadScoreFixture).toHaveBeenCalledTimes(1);
    expect(
      await screen.findByText(/Loaded Notation splitting\./i),
    ).toBeInTheDocument();
  });
});
