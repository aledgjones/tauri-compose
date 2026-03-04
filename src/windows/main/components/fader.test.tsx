import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Fader } from "./fader";

describe("Fader", () => {
  it("renders without crashing", () => {
    const { container } = render(<Fader volume={50} onChange={vi.fn()} />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it("increments volume by 1 on ArrowUp key", () => {
    const onChange = vi.fn();
    render(<Fader volume={50} onChange={onChange} />);
    const button = screen.getByRole("button");
    fireEvent.keyDown(button, { key: "ArrowUp" });
    expect(onChange).toHaveBeenCalledWith(51);
  });

  it("decrements volume by 1 on ArrowDown key", () => {
    const onChange = vi.fn();
    render(<Fader volume={50} onChange={onChange} />);
    const button = screen.getByRole("button");
    fireEvent.keyDown(button, { key: "ArrowDown" });
    expect(onChange).toHaveBeenCalledWith(49);
  });

  it("clamps at 100 on ArrowUp when volume is 100", () => {
    const onChange = vi.fn();
    render(<Fader volume={100} onChange={onChange} />);
    const button = screen.getByRole("button");
    fireEvent.keyDown(button, { key: "ArrowUp" });
    expect(onChange).toHaveBeenCalledWith(100);
  });

  it("clamps at 0 on ArrowDown when volume is 0", () => {
    const onChange = vi.fn();
    render(<Fader volume={0} onChange={onChange} />);
    const button = screen.getByRole("button");
    fireEvent.keyDown(button, { key: "ArrowDown" });
    expect(onChange).toHaveBeenCalledWith(0);
  });

  it("does not call onChange when disabled", () => {
    const onChange = vi.fn();
    render(<Fader volume={50} onChange={onChange} disabled />);
    const button = screen.getByRole("button");
    fireEvent.keyDown(button, { key: "ArrowUp" });
    expect(onChange).not.toHaveBeenCalled();
  });
});
