import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Checkbox } from "./checkbox";

describe("Checkbox", () => {
  it("renders without crashing", () => {
    const onChange = vi.fn();
    render(<Checkbox value={false} onChange={onChange} />);
    expect(screen.getByRole("checkbox")).toBeInTheDocument();
  });

  it("renders a check icon (svg) when value=true", () => {
    const { container } = render(<Checkbox value={true} onChange={vi.fn()} />);
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("does not render a check icon when value=false", () => {
    const { container } = render(<Checkbox value={false} onChange={vi.fn()} />);
    expect(container.querySelector("svg")).not.toBeInTheDocument();
  });

  it("calls onChange with true when clicked and value=false", () => {
    const onChange = vi.fn();
    render(<Checkbox value={false} onChange={onChange} />);
    fireEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledOnce();
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("calls onChange with false when clicked and value=true", () => {
    const onChange = vi.fn();
    render(<Checkbox value={true} onChange={onChange} />);
    fireEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledWith(false);
  });

  it("applies custom className to the button", () => {
    const { container } = render(
      <Checkbox value={false} onChange={vi.fn()} className="my-custom-class" />,
    );
    const button = container.querySelector("button");
    expect(button?.className).toContain("my-custom-class");
  });

  it("calls onPointerDown when provided", () => {
    const onPointerDown = vi.fn();
    render(
      <Checkbox
        value={false}
        onChange={vi.fn()}
        onPointerDown={onPointerDown}
      />,
    );
    fireEvent.pointerDown(screen.getByRole("checkbox"));
    expect(onPointerDown).toHaveBeenCalledOnce();
  });

  it("renders with role=checkbox and aria-checked reflecting value", () => {
    const { rerender } = render(<Checkbox value={true} onChange={vi.fn()} />);
    expect(screen.getByRole("checkbox", { checked: true })).toBeInTheDocument();

    rerender(<Checkbox value={false} onChange={vi.fn()} />);
    expect(
      screen.getByRole("checkbox", { checked: false }),
    ).toBeInTheDocument();
  });
});
