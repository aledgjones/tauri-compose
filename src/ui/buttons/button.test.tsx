import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Button } from "./button";

describe("Button", () => {
  it("renders children text", () => {
    render(<Button onClick={vi.fn()}>Click me</Button>);
    expect(screen.getByText("Click me")).toBeInTheDocument();
  });

  it("fires onClick when clicked", () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Action</Button>);
    fireEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("applies opacity-25 class to inner div when isDisabled=true", () => {
    const { container } = render(
      <Button onClick={vi.fn()} isDisabled>
        Label
      </Button>,
    );
    const inner = container.querySelector("div");
    expect(inner?.className).toContain("opacity-25");
  });

  it("does not apply opacity-25 when isDisabled is not set", () => {
    const { container } = render(<Button onClick={vi.fn()}>Label</Button>);
    const inner = container.querySelector("div");
    expect(inner?.className ?? "").not.toContain("opacity-25");
  });

  it("applies custom className to the button element", () => {
    const { container } = render(
      <Button onClick={vi.fn()} className="my-custom">
        Label
      </Button>,
    );
    expect(container.querySelector("button")?.className).toContain("my-custom");
  });
});
