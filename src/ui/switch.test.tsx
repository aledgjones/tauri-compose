import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Switch } from "./switch";

describe("Switch", () => {
  it("renders with role='switch'", () => {
    render(<Switch value={false} onChange={vi.fn()} />);
    expect(screen.getByRole("switch")).toBeInTheDocument();
  });

  it("aria-checked is true when value=true", () => {
    render(<Switch value={true} onChange={vi.fn()} />);
    expect(screen.getByRole("switch")).toHaveAttribute("aria-checked", "true");
  });

  it("aria-checked is false when value=false", () => {
    render(<Switch value={false} onChange={vi.fn()} />);
    expect(screen.getByRole("switch")).toHaveAttribute("aria-checked", "false");
  });

  it("aria-label reflects label prop", () => {
    render(<Switch value={false} onChange={vi.fn()} label="Dark mode" />);
    expect(screen.getByRole("switch")).toHaveAttribute(
      "aria-label",
      "Dark mode",
    );
  });

  it("calls onChange with true when clicked and value=false", () => {
    const onChange = vi.fn();
    render(<Switch value={false} onChange={onChange} />);
    fireEvent.click(screen.getByRole("switch"));
    expect(onChange).toHaveBeenCalledOnce();
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("calls onChange with false when clicked and value=true", () => {
    const onChange = vi.fn();
    render(<Switch value={true} onChange={onChange} />);
    fireEvent.click(screen.getByRole("switch"));
    expect(onChange).toHaveBeenCalledWith(false);
  });

  it("renders label text when provided", () => {
    render(<Switch value={false} onChange={vi.fn()} label="Enable feature" />);
    expect(screen.getByText("Enable feature")).toBeInTheDocument();
  });

  it("renders description text when provided", () => {
    render(
      <Switch
        value={false}
        onChange={vi.fn()}
        description="This changes the setting"
      />,
    );
    expect(screen.getByText("This changes the setting")).toBeInTheDocument();
  });

  it("calls onPointerDown when provided", () => {
    const onPointerDown = vi.fn();
    render(
      <Switch value={false} onChange={vi.fn()} onPointerDown={onPointerDown} />,
    );
    fireEvent.pointerDown(screen.getByRole("switch"));
    expect(onPointerDown).toHaveBeenCalledOnce();
  });

  it("does not call onChange when disabled", () => {
    const onChange = vi.fn();
    render(<Switch value={false} onChange={onChange} disabled />);
    fireEvent.click(screen.getByRole("switch"));
    expect(onChange).not.toHaveBeenCalled();
  });
});
