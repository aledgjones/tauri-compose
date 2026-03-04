import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Tools } from "./tools";

// ---------------------------------------------------------------------------
// Mock Tauri WebviewWindow — vi.hoisted ensures variables are available
// before the hoisted vi.mock factory runs.
// ---------------------------------------------------------------------------
const { MockWebviewWindow } = vi.hoisted(() => {
  // Must use a regular function (not arrow) so it can be called with `new`
  const MockWebviewWindow = vi.fn(function (this: Record<string, unknown>) {
    this.setFocus = vi.fn().mockResolvedValue(undefined);
  });
  return { MockWebviewWindow };
});

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: MockWebviewWindow,
}));

describe("Tools", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing", () => {
    const { container } = render(<Tools />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it("renders exactly 4 icon buttons", () => {
    render(<Tools />);
    expect(screen.getAllByRole("button")).toHaveLength(4);
  });

  it("undo button has pointer-events-none (isDisabled)", () => {
    render(<Tools />);
    const buttons = screen.getAllByRole("button");
    // undo is 3rd button (index 2)
    expect(buttons[2].className).toContain("pointer-events-none");
  });

  it("redo button has pointer-events-none (isDisabled)", () => {
    render(<Tools />);
    const buttons = screen.getAllByRole("button");
    // redo is 4th button (index 3)
    expect(buttons[3].className).toContain("pointer-events-none");
  });

  it("clicking project-settings button creates a WebviewWindow with id 'project-settings'", () => {
    render(<Tools />);
    const buttons = screen.getAllByRole("button");
    // project-settings is the 1st button (index 0)
    fireEvent.click(buttons[0]);
    expect(MockWebviewWindow).toHaveBeenCalledWith(
      "project-settings",
      expect.objectContaining({ url: "/project-settings.html" }),
    );
  });

  it("clicking debugger button creates a WebviewWindow with id 'debugger'", () => {
    render(<Tools />);
    const buttons = screen.getAllByRole("button");
    // debugger is the 2nd button (index 1)
    fireEvent.click(buttons[1]);
    expect(MockWebviewWindow).toHaveBeenCalledWith(
      "debugger",
      expect.objectContaining({ url: "/debugger.html" }),
    );
  });

  it("applies custom className to the container", () => {
    const { container } = render(<Tools className="my-tools" />);
    expect((container.firstChild as HTMLElement).className).toContain(
      "my-tools",
    );
  });
});
