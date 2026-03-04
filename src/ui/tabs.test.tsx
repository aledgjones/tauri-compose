import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Tabs } from "./tabs";
import type { TabsOption } from "./tabs";

const STRING_OPTIONS: TabsOption<string>[] = [
  { value: "alpha", label: "Alpha" },
  { value: "beta", label: "Beta" },
  { value: "gamma", label: "Gamma" },
];

const NUMBER_OPTIONS: TabsOption<number>[] = [
  { value: 1, label: "One" },
  { value: 2, label: "Two" },
  { value: 3, label: "Three" },
];

describe("Tabs", () => {
  it("renders all string options as buttons", () => {
    render(<Tabs value="alpha" options={STRING_OPTIONS} onChange={vi.fn()} />);
    expect(screen.getByText("Alpha")).toBeInTheDocument();
    expect(screen.getByText("Beta")).toBeInTheDocument();
    expect(screen.getByText("Gamma")).toBeInTheDocument();
  });

  it("selected tab has active styling class", () => {
    render(<Tabs value="beta" options={STRING_OPTIONS} onChange={vi.fn()} />);
    const betaButton = screen.getByText("Beta").closest("button");
    expect(betaButton?.className).toContain("bg-sky-500");
    expect(betaButton?.className).toContain("text-black");
  });

  it("non-selected tabs do not have active styling", () => {
    render(<Tabs value="alpha" options={STRING_OPTIONS} onChange={vi.fn()} />);
    const betaButton = screen.getByText("Beta").closest("button");
    expect(betaButton?.className).not.toContain("bg-sky-500");
  });

  it("clicking a tab fires onChange with the option's value", () => {
    const onChange = vi.fn();
    render(<Tabs value="alpha" options={STRING_OPTIONS} onChange={onChange} />);
    fireEvent.click(screen.getByText("Beta"));
    expect(onChange).toHaveBeenCalledOnce();
    expect(onChange).toHaveBeenCalledWith("beta");
  });

  it("works with number value type", () => {
    const onChange = vi.fn();
    render(<Tabs value={1} options={NUMBER_OPTIONS} onChange={onChange} />);
    expect(screen.getByText("One")).toBeInTheDocument();
    fireEvent.click(screen.getByText("Three"));
    expect(onChange).toHaveBeenCalledWith(3);
  });

  it("correctly identifies selected tab among number options", () => {
    render(<Tabs value={2} options={NUMBER_OPTIONS} onChange={vi.fn()} />);
    const twoButton = screen.getByText("Two").closest("button");
    expect(twoButton?.className).toContain("bg-sky-500");
  });
});
