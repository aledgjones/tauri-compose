import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SnapPicker } from "./snap-picker";
import { NoteDuration } from "../services/types";

const SNAP_OPTIONS_LABELS = ["Quarter", "Eighth", "Sixteenth", "Thirty-Second"];

describe("SnapPicker", () => {
  it("renders without crashing", () => {
    render(
      <SnapPicker
        variant="standard"
        value={NoteDuration.Quarter}
        onChange={vi.fn()}
      />,
    );
    expect(screen.getByRole("button")).toBeInTheDocument();
  });

  it("shows the current value's label in the trigger button", () => {
    render(
      <SnapPicker
        variant="standard"
        value={NoteDuration.Eighth}
        onChange={vi.fn()}
      />,
    );
    expect(screen.getByText("Eighth")).toBeInTheDocument();
  });

  it("opens the dropdown and shows all 4 snap options", async () => {
    const user = userEvent.setup();
    render(
      <SnapPicker
        variant="standard"
        value={NoteDuration.Quarter}
        onChange={vi.fn()}
      />,
    );
    await user.click(screen.getByRole("button"));
    expect(screen.getAllByRole("option")).toHaveLength(4);
    for (const label of SNAP_OPTIONS_LABELS) {
      expect(screen.getByRole("option", { name: label })).toBeInTheDocument();
    }
  });

  it("fires onChange with NoteDuration.Sixteenth when Sixteenth is selected", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <SnapPicker
        variant="standard"
        value={NoteDuration.Quarter}
        onChange={onChange}
      />,
    );
    await user.click(screen.getByRole("button"));
    await user.click(screen.getByText("Sixteenth"));
    expect(onChange).toHaveBeenCalledOnce();
    expect(onChange).toHaveBeenCalledWith(NoteDuration.Sixteenth);
  });

  it("fires onChange with NoteDuration.ThirtySecond when Thirty-Second is selected", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <SnapPicker
        variant="standard"
        value={NoteDuration.Quarter}
        onChange={onChange}
      />,
    );
    await user.click(screen.getByRole("button"));
    await user.click(screen.getByText("Thirty-Second"));
    expect(onChange).toHaveBeenCalledWith(NoteDuration.ThirtySecond);
  });

  it("renders optional label when provided", () => {
    render(
      <SnapPicker
        variant="standard"
        value={NoteDuration.Quarter}
        onChange={vi.fn()}
        label="Snap Grid"
      />,
    );
    expect(screen.getByText("Snap Grid")).toBeInTheDocument();
  });
});
