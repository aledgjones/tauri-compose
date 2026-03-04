import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { DragScroll } from "./drag-scroll";

describe("DragScroll", () => {
  it("renders children", () => {
    render(
      <DragScroll>
        <p>Inner content</p>
      </DragScroll>,
    );
    expect(screen.getByText("Inner content")).toBeInTheDocument();
  });

  it("fires onScroll callback when scrolled", () => {
    const onScroll = vi.fn();
    const { container } = render(
      <DragScroll onScroll={onScroll}>
        <div style={{ height: "1000px" }} />
      </DragScroll>,
    );
    const div = container.firstChild as HTMLElement;
    // Simulate a scroll event with expected scrollLeft/scrollTop
    Object.defineProperty(div, "scrollLeft", { value: 50, writable: true });
    Object.defineProperty(div, "scrollTop", { value: 100, writable: true });
    fireEvent.scroll(div);
    expect(onScroll).toHaveBeenCalledWith(50, 100);
  });

  it("does not throw when onScroll is not provided", () => {
    const { container } = render(
      <DragScroll>
        <div />
      </DragScroll>,
    );
    const div = container.firstChild as HTMLElement;
    expect(() => fireEvent.scroll(div)).not.toThrow();
  });

  it("applies custom className", () => {
    const { container } = render(
      <DragScroll className="my-class">
        <div />
      </DragScroll>,
    );
    expect((container.firstChild as HTMLElement).className).toContain(
      "my-class",
    );
  });
});
