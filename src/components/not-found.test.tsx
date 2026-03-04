import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { NotFound } from "./not-found";

describe("NotFound", () => {
  it('renders "Nothing here, yet..." heading', () => {
    render(<NotFound />);
    expect(screen.getByText("Nothing here, yet...")).toBeInTheDocument();
  });

  it("renders the secondary message", () => {
    render(<NotFound />);
    expect(
      screen.getByText(/I'll get to this at some point/i),
    ).toBeInTheDocument();
  });
});
