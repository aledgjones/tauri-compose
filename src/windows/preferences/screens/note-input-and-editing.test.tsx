import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { NoteInputAndEditing } from "./note-input-and-editing";
import {
  renderWithProviders,
  createAppState,
} from "../../../services/test-utils";

vi.mock("../../../services/actions", () => ({
  actions: {
    ui: {
      preferences: {
        setAudition: vi.fn(),
      },
    },
  },
}));

import { actions } from "../../../services/actions";

describe("NoteInputAndEditing", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the Auditioning heading", () => {
    renderWithProviders(<NoteInputAndEditing />);
    expect(screen.getByText("Auditioning")).toBeInTheDocument();
  });

  it("renders the audition switch label", () => {
    renderWithProviders(<NoteInputAndEditing />);
    expect(
      screen.getByText("Play notes during note input and or modification"),
    ).toBeInTheDocument();
  });

  it("renders the switch in the on state when audition is true", () => {
    const appState = createAppState({
      preferences: { audition: true },
    } as never);
    renderWithProviders(<NoteInputAndEditing />, { appState });
    const switchEl = screen.getByRole("switch");
    expect(switchEl).toHaveAttribute("aria-checked", "true");
  });

  it("renders the switch in the off state when audition is false", () => {
    const appState = createAppState({
      preferences: { audition: false },
    } as never);
    renderWithProviders(<NoteInputAndEditing />, { appState });
    const switchEl = screen.getByRole("switch");
    expect(switchEl).toHaveAttribute("aria-checked", "false");
  });

  it("calls actions.ui.preferences.setAudition when switch is toggled", () => {
    renderWithProviders(<NoteInputAndEditing />);
    const switchEl = screen.getByRole("switch");
    fireEvent.click(switchEl);
    expect(actions.ui.preferences.setAudition).toHaveBeenCalledOnce();
  });
});
