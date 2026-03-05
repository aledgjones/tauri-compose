import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { mdiClose, mdiHome, mdiMagnify } from "@mdi/js";
import { Icon } from "./icon";

describe("Icon", () => {
  it("renders without crashing", () => {
    const { container } = render(<Icon path={mdiClose} />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it("wraps the MdiIcon in a div", () => {
    const { container } = render(<Icon path={mdiClose} />);
    expect(container.firstChild?.nodeName).toBe("DIV");
  });

  it("applies className to the wrapper div", () => {
    const { container } = render(
      <Icon path={mdiClose} className="my-custom-class" />,
    );
    expect(container.firstChild).toHaveClass("my-custom-class");
  });

  it("renders an SVG element inside", () => {
    const { container } = render(<Icon path={mdiHome} />);
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("uses default size of 1 when no size prop given", () => {
    const { container } = render(<Icon path={mdiClose} />);
    const svg = container.querySelector("svg") as SVGElement;
    // @mdi/react converts size to rem: size=1 → 1.5rem
    expect(svg.style.width).toBe("1.5rem");
    expect(svg.style.height).toBe("1.5rem");
  });

  it("renders with an explicit size prop", () => {
    const { container } = render(<Icon path={mdiMagnify} size={2} />);
    const svg = container.querySelector("svg") as SVGElement;
    // size=2 → 3rem
    expect(svg.style.width).toBe("3rem");
    expect(svg.style.height).toBe("3rem");
  });

  it("renders with no className when omitted", () => {
    const { container } = render(<Icon path={mdiClose} />);
    const wrapper = container.firstChild as HTMLElement;
    // className attribute should be absent or empty
    expect(wrapper.className).toBe("");
  });
});
