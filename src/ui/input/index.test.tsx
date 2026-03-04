import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { Input } from "./index";

describe("Input", () => {
  it("renders a text input when type=text", () => {
    render(<Input type="text" value="hello" onChange={vi.fn()} />);
    const input = screen.getByRole("textbox");
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue("hello");
  });

  it("renders a number input when type=number", () => {
    render(
      <Input
        type="number"
        value={42}
        onChange={vi.fn()}
        step={1}
        decimalPlaces={0}
      />,
    );
    const input = screen.getByRole("textbox");
    expect(input).toBeInTheDocument();
    expect(input).toHaveValue("42");
  });
});
