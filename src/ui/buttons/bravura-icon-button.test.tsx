import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { BravuraIconButton } from "./bravura-icon-button";
import { SMuFL } from "./smufl";

describe("BravuraIconButton", () => {
  it("renders without crashing with a SMuFL glyph", () => {
    render(<BravuraIconButton bravura={SMuFL.Staccato} onClick={vi.fn()} />);
    expect(screen.getByRole("button")).toBeInTheDocument();
  });

  it("fires onClick when clicked", () => {
    const onClick = vi.fn();
    render(<BravuraIconButton bravura={SMuFL.Accent} onClick={onClick} />);
    fireEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("sets title attribute from title prop", () => {
    render(
      <BravuraIconButton
        bravura={SMuFL.Tenuto}
        title="Tenuto"
        onClick={vi.fn()}
      />,
    );
    expect(screen.getByRole("button")).toHaveAttribute("title", "Tenuto");
  });

  it("applies pointer-events-none when isDisabled=true", () => {
    render(
      <BravuraIconButton
        bravura={SMuFL.Staccato}
        onClick={vi.fn()}
        isDisabled
      />,
    );
    expect(screen.getByRole("button").className).toContain(
      "pointer-events-none",
    );
  });

  it("does not apply pointer-events-none when not disabled", () => {
    render(<BravuraIconButton bravura={SMuFL.Staccato} onClick={vi.fn()} />);
    expect(screen.getByRole("button").className).not.toContain(
      "pointer-events-none",
    );
  });

  it("applies selected theme when isSelected=true", () => {
    render(
      <BravuraIconButton
        bravura={SMuFL.Staccato}
        onClick={vi.fn()}
        isSelected
      />,
    );
    expect(screen.getByRole("button").className).toContain("bg-sky-500");
  });

  it("renders glyph character in a font-bravura span", () => {
    const { container } = render(
      <BravuraIconButton bravura={SMuFL.Sharp} onClick={vi.fn()} />,
    );
    const span = container.querySelector("span.font-bravura");
    expect(span).toBeInTheDocument();
    expect(span?.textContent).toBe(SMuFL.Sharp);
  });
});
