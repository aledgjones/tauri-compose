import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { mdiClose } from "@mdi/js";
import { IconButton } from "./icon-button";

describe("IconButton", () => {
  it("renders without crashing", () => {
    render(
      <IconButton variant="interactive" path={mdiClose} onClick={vi.fn()} />,
    );
    expect(screen.getByRole("button")).toBeInTheDocument();
  });

  it("fires onClick when clicked", () => {
    const onClick = vi.fn();
    render(
      <IconButton variant="interactive" path={mdiClose} onClick={onClick} />,
    );
    fireEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("applies pointer-events-none when isDisabled=true", () => {
    render(
      <IconButton
        variant="interactive"
        path={mdiClose}
        onClick={vi.fn()}
        isDisabled
      />,
    );
    expect(screen.getByRole("button").className).toContain(
      "pointer-events-none",
    );
  });

  it("does not apply pointer-events-none when not disabled", () => {
    render(
      <IconButton variant="interactive" path={mdiClose} onClick={vi.fn()} />,
    );
    expect(screen.getByRole("button").className).not.toContain(
      "pointer-events-none",
    );
  });

  it("applies selected theme when isSelected=true", () => {
    render(
      <IconButton
        variant="interactive"
        path={mdiClose}
        onClick={vi.fn()}
        isSelected
      />,
    );
    expect(screen.getByRole("button").className).toContain("bg-sky-500");
  });

  it("applies variant class when not selected", () => {
    render(<IconButton variant="filled" path={mdiClose} onClick={vi.fn()} />);
    expect(screen.getByRole("button").className).toContain("bg-neutral-800");
  });

  it("renders an SVG icon", () => {
    const { container } = render(
      <IconButton variant="interactive" path={mdiClose} onClick={vi.fn()} />,
    );
    expect(container.querySelector("svg")).toBeInTheDocument();
  });
});
