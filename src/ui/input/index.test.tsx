import { describe, it, expect, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
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

  it("keeps number validation on blur", () => {
    const onChange = vi.fn();

    render(
      <Input
        type="number"
        value={42}
        onChange={onChange}
        step={1}
        decimalPlaces={0}
      />,
    );

    const input = screen.getByRole("textbox");

    fireEvent.change(input, { target: { value: "43" } });
    expect(onChange).not.toHaveBeenCalled();

    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith(43);
  });

  it("updates a focused number input when ArrowUp changes the external value", () => {
    const ControlledNumberInput = () => {
      const [value, setValue] = useState(42);

      return (
        <Input
          type="number"
          value={value}
          onChange={setValue}
          step={1}
          decimalPlaces={0}
        />
      );
    };

    render(<ControlledNumberInput />);

    const input = screen.getByRole("textbox");
    fireEvent.focus(input);
    fireEvent.keyDown(input, { key: "ArrowUp" });

    expect(input).toHaveValue("43");
  });
});
