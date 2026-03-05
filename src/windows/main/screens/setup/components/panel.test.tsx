import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Panel } from "./panel";

describe("Panel", () => {
  it("renders the title", () => {
    render(<Panel title="Players">content</Panel>);
    expect(screen.getByText("Players")).toBeInTheDocument();
  });

  it("renders children", () => {
    render(
      <Panel title="Test">
        <span>child content</span>
      </Panel>,
    );
    expect(screen.getByText("child content")).toBeInTheDocument();
  });

  it("renders icon buttons when icons prop is provided", () => {
    const onClick = vi.fn();
    render(
      <Panel
        title="Flows"
        icons={[{ key: "add", path: "M0 0h24v24H0z", onClick }]}
      >
        content
      </Panel>,
    );
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(1);
  });

  it("calls the icon onClick when button is clicked", async () => {
    const onClick = vi.fn();
    render(
      <Panel
        title="Flows"
        icons={[{ key: "add", path: "M0 0h24v24H0z", onClick }]}
      >
        content
      </Panel>,
    );
    const [button] = screen.getAllByRole("button");
    await userEvent.click(button);
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("renders without icons when icons prop is omitted", () => {
    render(<Panel title="Layouts">body</Panel>);
    expect(screen.queryAllByRole("button")).toHaveLength(0);
  });

  it("applies extra className to the container", () => {
    const { container } = render(
      <Panel title="X" className="row-span-2">
        body
      </Panel>,
    );
    expect(container.firstChild).toHaveClass("row-span-2");
  });
});
