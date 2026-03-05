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

  it("handles pointerDown and registers pointermove/pointerup listeners", () => {
    const { container } = render(
      <DragScroll>
        <div />
      </DragScroll>,
    );
    const div = container.firstChild as HTMLElement;
    // Should not throw when pointer is pressed inside the component
    expect(() => {
      fireEvent.pointerDown(div, { clientX: 100, clientY: 100 });
    }).not.toThrow();
  });

  it("scrolls when pointer is dragged after pointerDown", () => {
    const { container } = render(
      <DragScroll>
        <div style={{ width: "2000px", height: "2000px" }} />
      </DragScroll>,
    );
    const div = container.firstChild as HTMLElement;

    // Define writable scrollLeft/scrollTop
    let scrollLeft = 0;
    let scrollTop = 0;
    Object.defineProperty(div, "scrollLeft", {
      get: () => scrollLeft,
      set: (v) => {
        scrollLeft = v;
      },
    });
    Object.defineProperty(div, "scrollTop", {
      get: () => scrollTop,
      set: (v) => {
        scrollTop = v;
      },
    });

    fireEvent.pointerDown(div, { clientX: 200, clientY: 200 });

    // Move 50px right and 30px down
    fireEvent.pointerMove(document, { clientX: 150, clientY: 170 });

    // scrollLeft should have increased (moved right by subtraction = 200-150=50)
    expect(scrollLeft).toBe(50);
    expect(scrollTop).toBe(30);
  });

  it("stops scrolling after pointerUp", () => {
    const { container } = render(
      <DragScroll>
        <div style={{ width: "2000px", height: "2000px" }} />
      </DragScroll>,
    );
    const div = container.firstChild as HTMLElement;

    let scrollLeft = 0;
    Object.defineProperty(div, "scrollLeft", {
      get: () => scrollLeft,
      set: (v) => {
        scrollLeft = v;
      },
    });

    fireEvent.pointerDown(div, { clientX: 200, clientY: 200 });
    fireEvent.pointerUp(document);

    // After pointerUp, moves should no longer update scroll
    fireEvent.pointerMove(document, { clientX: 100, clientY: 100 });
    expect(scrollLeft).toBe(0);
  });

  it("allow() returns false for elements with matching ignore class", () => {
    const { container } = render(
      <DragScroll>
        <div className="ignore-x">
          <button id="inner-btn">click</button>
        </div>
      </DragScroll>,
    );
    const div = container.firstChild as HTMLElement;
    const ignoreEl = container.querySelector(".ignore-x") as HTMLElement;

    let scrollLeft = 0;
    Object.defineProperty(div, "scrollLeft", {
      get: () => scrollLeft,
      set: (v) => {
        scrollLeft = v;
      },
    });

    // pointerDown on an element inside ignore-x class — x scroll should be blocked
    fireEvent.pointerDown(ignoreEl, { clientX: 100, clientY: 100 });
    fireEvent.pointerMove(document, { clientX: 50, clientY: 100 });

    // x scroll blocked, so scrollLeft stays 0
    expect(scrollLeft).toBe(0);
  });
});
