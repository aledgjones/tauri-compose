import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Menu } from "./menu";
import type { MenuItem } from "./menu";

const items: MenuItem<string>[] = [
  { label: "Option A", value: "a" },
  { label: "Option B", value: "b" },
  {
    label: "Option C",
    value: "c",
    sections: [{ label: "Section 1", value: "a" }],
  },
];

describe("Menu", () => {
  it("renders all menu items", () => {
    render(<Menu items={items} selection="a" onSelect={vi.fn()} />);
    expect(screen.getByText("Option A")).toBeInTheDocument();
    expect(screen.getByText("Option B")).toBeInTheDocument();
    expect(screen.getByText("Option C")).toBeInTheDocument();
  });

  it("selected item has active border styling", () => {
    render(<Menu items={items} selection="b" onSelect={vi.fn()} />);
    const button = screen.getByText("Option B").closest("button")!;
    expect(button.className).toContain("bg-neutral-800");
    expect(button.className).toContain("border-sky-500");
  });

  it("unselected item has transparent border", () => {
    render(<Menu items={items} selection="b" onSelect={vi.fn()} />);
    const button = screen.getByText("Option A").closest("button")!;
    expect(button.className).toContain("border-transparent");
  });

  it("fires onSelect with correct value when item is clicked", () => {
    const onSelect = vi.fn();
    render(<Menu items={items} selection="a" onSelect={onSelect} />);
    fireEvent.click(screen.getByText("Option B"));
    expect(onSelect).toHaveBeenCalledWith("b");
  });

  it("renders sections when selected item has them", () => {
    render(<Menu items={items} selection="c" onSelect={vi.fn()} />);
    expect(screen.getByText("Section 1")).toBeInTheDocument();
  });

  it("does not render sections for non-selected items", () => {
    render(<Menu items={items} selection="a" onSelect={vi.fn()} />);
    expect(screen.queryByText("Section 1")).not.toBeInTheDocument();
  });
});
