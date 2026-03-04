import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { SelectByImage } from "./select-by-image";
import type { SelectByImageOption } from "./select-by-image";

const options: SelectByImageOption<string>[] = [
  { value: "a", label: "Option A", image: "/img/a.png" },
  { value: "b", label: "Option B", image: "/img/b.png" },
  { value: "c", label: "Option C", image: "/img/c.png" },
];

describe("SelectByImage", () => {
  it("renders all options", () => {
    render(<SelectByImage value="a" options={options} onChange={vi.fn()} />);
    expect(screen.getByText("Option A")).toBeInTheDocument();
    expect(screen.getByText("Option B")).toBeInTheDocument();
    expect(screen.getByText("Option C")).toBeInTheDocument();
  });

  it("renders an image for each option", () => {
    const { container } = render(
      <SelectByImage value="a" options={options} onChange={vi.fn()} />,
    );
    const images = container.querySelectorAll("img");
    expect(images).toHaveLength(3);
  });

  it("selected option has border-sky-500 class on its image", () => {
    const { container } = render(
      <SelectByImage value="b" options={options} onChange={vi.fn()} />,
    );
    const images = container.querySelectorAll("img");
    // images are ordered same as options: a=0, b=1, c=2
    expect(images[1].className).toContain("border-sky-500");
  });

  it("non-selected option has border-white on its image", () => {
    const { container } = render(
      <SelectByImage value="b" options={options} onChange={vi.fn()} />,
    );
    const images = container.querySelectorAll("img");
    expect(images[0].className).toContain("border-white");
  });

  it("fires onChange with correct value when option is clicked", () => {
    const onChange = vi.fn();
    render(<SelectByImage value="a" options={options} onChange={onChange} />);
    fireEvent.click(screen.getByText("Option C").closest("button")!);
    expect(onChange).toHaveBeenCalledWith("c");
  });

  it("renders label text when provided", () => {
    render(
      <SelectByImage
        value="a"
        options={options}
        onChange={vi.fn()}
        label="Choose style"
      />,
    );
    expect(screen.getByText("Choose style")).toBeInTheDocument();
  });
});
