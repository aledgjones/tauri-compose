import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { Tools } from "./tools";

vi.mock("@tauri-apps/api/webviewWindow", () => ({
  WebviewWindow: vi.fn().mockImplementation(() => ({
    setFocus: vi.fn().mockResolvedValue(undefined),
  })),
}));

describe("Tools", () => {
  it("renders without crashing", () => {
    const { container } = render(<Tools />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it("renders toolbar buttons", () => {
    render(<Tools />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThan(0);
  });
});
