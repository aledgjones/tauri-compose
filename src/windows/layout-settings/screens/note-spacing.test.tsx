import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen } from "@testing-library/react";
import { NoteSpacing } from "./note-spacing";
import { renderWithProviders } from "../../../services/test-utils";
import { LayoutType } from "../../../services/types";

vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      engrave: {
        setBaseNoteSpace: vi.fn(),
        setMinimumNoteSpace: vi.fn(),
        setMinimumTieSpace: vi.fn(),
        setNoteSpaceRatio: vi.fn(),
      },
    },
  },
}));

describe("NoteSpacing", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the Note Spacing heading", () => {
    renderWithProviders(<NoteSpacing layoutType={LayoutType.FullScore} />);
    expect(screen.getByText("Note Spacing")).toBeInTheDocument();
  });

  it("renders all four input labels", () => {
    renderWithProviders(<NoteSpacing layoutType={LayoutType.FullScore} />);
    expect(screen.getByText("Base Note Space")).toBeInTheDocument();
    expect(screen.getByText("Minimum Note Space")).toBeInTheDocument();
    expect(screen.getByText("Minimum Tie Space")).toBeInTheDocument();
    expect(screen.getByText("Note Space Ratio")).toBeInTheDocument();
  });

  it("renders Part layoutType without crashing", () => {
    renderWithProviders(<NoteSpacing layoutType={LayoutType.Part} />);
    expect(screen.getByText("Note Spacing")).toBeInTheDocument();
  });
});
