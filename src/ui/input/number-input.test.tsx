import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { NumberInput } from "./number-input";

describe("NumberInput", () => {
  it("renders value with correct decimal places", () => {
    render(
      <NumberInput
        type="number"
        value={1}
        onChange={vi.fn()}
        step={0.1}
        decimalPlaces={2}
      />,
    );
    expect(screen.getByRole("textbox")).toHaveValue("1.00");
  });

  it("increments by step on ArrowUp", () => {
    const onChange = vi.fn();
    render(
      <NumberInput
        type="number"
        value={5}
        onChange={onChange}
        step={1}
        decimalPlaces={0}
      />,
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "ArrowUp" });
    expect(onChange).toHaveBeenCalledWith(6);
  });

  it("decrements by step on ArrowDown", () => {
    const onChange = vi.fn();
    render(
      <NumberInput
        type="number"
        value={5}
        onChange={onChange}
        step={1}
        decimalPlaces={0}
      />,
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "ArrowDown" });
    expect(onChange).toHaveBeenCalledWith(4);
  });

  it("calls onChange with parsed number on valid blur", () => {
    const onChange = vi.fn();
    render(
      <NumberInput
        type="number"
        value={5}
        onChange={onChange}
        step={1}
        decimalPlaces={1}
      />,
    );
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "7.5" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith(7.5);
  });

  it("reverts to original value on invalid blur", () => {
    render(
      <NumberInput
        type="number"
        value={3}
        onChange={vi.fn()}
        step={1}
        decimalPlaces={1}
      />,
    );
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "not-a-number" } });
    fireEvent.blur(input);
    expect(input).toHaveValue("3.0");
  });

  it("applies readOnly to the underlying input", () => {
    render(
      <NumberInput
        type="number"
        value={1}
        onChange={vi.fn()}
        step={1}
        decimalPlaces={0}
        readOnly
      />,
    );
    expect(screen.getByRole("textbox")).toHaveAttribute("readonly");
  });

  it("does nothing on ArrowUp when value is undefined", () => {
    const onChange = vi.fn();
    render(
      <NumberInput
        type="number"
        value={undefined}
        onChange={onChange}
        step={1}
        decimalPlaces={0}
      />,
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "ArrowUp" });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("does nothing on ArrowDown when value is undefined", () => {
    const onChange = vi.fn();
    render(
      <NumberInput
        type="number"
        value={undefined}
        onChange={onChange}
        step={1}
        decimalPlaces={0}
      />,
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "ArrowDown" });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("does nothing on non-arrow key press (default switch case)", () => {
    const onChange = vi.fn();
    render(
      <NumberInput
        type="number"
        value={5}
        onChange={onChange}
        step={1}
        decimalPlaces={0}
      />,
    );
    fireEvent.keyDown(screen.getByRole("textbox"), { key: "Enter" });
    expect(onChange).not.toHaveBeenCalled();
  });

  it("falls back to the current value when blurred with empty display", () => {
    const onChange = vi.fn();
    render(
      <NumberInput
        type="number"
        value={5}
        onChange={onChange}
        step={1}
        decimalPlaces={0}
      />,
    );
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith(5);
  });

  it("uses the provided default value when blurred with an empty display", () => {
    const onChange = vi.fn();
    render(
      <NumberInput
        type="number"
        value={5}
        defaultValue={24}
        onChange={onChange}
        step={1}
        decimalPlaces={0}
      />,
    );
    const input = screen.getByRole("textbox");
    fireEvent.change(input, { target: { value: "" } });
    fireEvent.blur(input);
    expect(onChange).toHaveBeenCalledWith(24);
  });
});
