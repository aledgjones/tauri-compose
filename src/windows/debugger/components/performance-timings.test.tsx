import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PerformanceTimings } from "./performance-timings";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(vi.fn()),
}));

describe("PerformanceTimings", () => {
  it("renders all three section toggle buttons", () => {
    render(<PerformanceTimings />);
    expect(screen.getByText(/Layout Phase/)).toBeInTheDocument();
    expect(screen.getByText(/Render Phase/)).toBeInTheDocument();
    expect(screen.getByText(/GPU/)).toBeInTheDocument();
  });

  it("layout section starts collapsed (aria-expanded=false)", () => {
    render(<PerformanceTimings />);
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    expect(layoutBtn.getAttribute("aria-expanded")).toBe("false");
  });

  it("toggles layout section open on click", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    await user.click(layoutBtn);
    expect(layoutBtn.getAttribute("aria-expanded")).toBe("true");
  });

  it("toggles layout section closed on second click", async () => {
    const user = userEvent.setup();
    render(<PerformanceTimings />);
    const layoutBtn = screen.getByRole("button", { name: /Layout Phase/ });
    await user.click(layoutBtn);
    await user.click(layoutBtn);
    expect(layoutBtn.getAttribute("aria-expanded")).toBe("false");
  });
});
