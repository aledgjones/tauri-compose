import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { TickBar } from "./tick-bar";
import type { TickConfig } from "../hooks/use-ticks";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const emptyConfig: TickConfig = {
  tickWidth: 2.5,
  entries: [],
  length: 0,
};

const barOnlyConfig: TickConfig = {
  tickWidth: 2.5,
  entries: [{ x: 0, type: "bar", barNumber: 1 }],
  length: 128,
};

const mixedConfig: TickConfig = {
  tickWidth: 2.5,
  entries: [
    { x: 0, type: "bar", barNumber: 1 },
    { x: 80, type: "beat" },
    { x: 160, type: "beat" },
    { x: 240, type: "beat" },
    { x: 320, type: "bar", barNumber: 2 },
    { x: 40, type: "sub" },
  ],
  length: 512,
};

describe("TickBar", () => {
  it("renders without crashing with an empty config", () => {
    const { container } = render(<TickBar ticks={emptyConfig} />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it("applies the outer container width based on tickWidth × length", () => {
    const { container } = render(<TickBar ticks={barOnlyConfig} />);
    const wrapper = container.firstChild as HTMLElement;
    // tickWidth (2.5) * length (128) = 320
    expect(wrapper.style.width).toBe("320px");
  });

  it("sets width to 0 when length is 0", () => {
    const { container } = render(<TickBar ticks={emptyConfig} />);
    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper.style.width).toBe("0px");
  });

  it("applies an optional className to the wrapper", () => {
    const { container } = render(
      <TickBar ticks={emptyConfig} className="tick-bar-custom" />,
    );
    expect(container.firstChild).toHaveClass("tick-bar-custom");
  });

  it("always applies the h-7 class to the wrapper", () => {
    const { container } = render(<TickBar ticks={emptyConfig} />);
    expect(container.firstChild).toHaveClass("h-7");
  });

  it("renders a bar-number label for entries with barNumber", () => {
    render(<TickBar ticks={barOnlyConfig} />);
    expect(screen.getByText("1")).toBeInTheDocument();
  });

  it("renders multiple bar-number labels for multiple bar entries", () => {
    render(<TickBar ticks={mixedConfig} />);
    expect(screen.getByText("1")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("renders no labels for beat entries", () => {
    const beatOnlyConfig: TickConfig = {
      tickWidth: 2.5,
      entries: [
        { x: 80, type: "beat" },
        { x: 160, type: "beat" },
      ],
      length: 256,
    };
    render(<TickBar ticks={beatOnlyConfig} />);
    // No bar numbers — only empty ticks
    expect(screen.queryByText(/^\d+$/)).toBeNull();
  });

  it("renders tick marks with correct left positions", () => {
    const { container } = render(<TickBar ticks={barOnlyConfig} />);
    // The inner tick div should have left=0px (entry.x=0)
    const tickDiv = container.querySelector<HTMLElement>(".absolute.bottom-0")!;
    expect(tickDiv.style.left).toBe("0px");
  });

  it("renders a full-height tick for bar entries", () => {
    const { container } = render(<TickBar ticks={barOnlyConfig} />);
    const tickDiv = container.querySelector(".absolute.bottom-0");
    expect(tickDiv).toHaveClass("h-full");
  });

  it("renders 50%-height tick for beat entries", () => {
    const beatConfig: TickConfig = {
      tickWidth: 2.5,
      entries: [{ x: 80, type: "beat" }],
      length: 128,
    };
    const { container } = render(<TickBar ticks={beatConfig} />);
    const tickDiv = container.querySelector(".absolute.bottom-0");
    expect(tickDiv).toHaveClass("h-[50%]");
  });

  it("renders 25%-height tick for sub entries", () => {
    const subConfig: TickConfig = {
      tickWidth: 2.5,
      entries: [{ x: 40, type: "sub" }],
      length: 128,
    };
    const { container } = render(<TickBar ticks={subConfig} />);
    const tickDiv = container.querySelector(".absolute.bottom-0");
    expect(tickDiv).toHaveClass("h-[25%]");
  });

  it("renders all entry types in a mixed config", () => {
    const { container } = render(<TickBar ticks={mixedConfig} />);
    // Use the tick-mark divs specifically (absolute bottom-0 is always on tick divs)
    const tickDivs = container.querySelectorAll(".absolute.bottom-0");
    const barDivs = Array.from(tickDivs).filter((el) =>
      el.classList.contains("h-full"),
    );
    const beatDivs = Array.from(tickDivs).filter((el) =>
      el.className.includes("h-[50%]"),
    );
    const subDivs = Array.from(tickDivs).filter((el) =>
      el.className.includes("h-[25%]"),
    );
    expect(barDivs).toHaveLength(2);
    expect(beatDivs).toHaveLength(3);
    expect(subDivs).toHaveLength(1);
  });
});
