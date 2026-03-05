import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Popover } from "./popover";
import type { PopoverState } from "./popover";
import { renderWithProviders, createAppState } from "../services/test-utils";
import { PopoverType } from "../services/popovers";

// ---------------------------------------------------------------------------
// Mock actions and popovers so we don't invoke real IPC or parse logic
// ---------------------------------------------------------------------------
vi.mock("../services/actions", () => ({
  actions: {
    score: {
      barline: { create: vi.fn(), clearAtTick: vi.fn() },
      clef: { create: vi.fn(), clear: vi.fn() },
      keySignature: { create: vi.fn(), clearAtTick: vi.fn() },
      timeSignature: { create: vi.fn(), clearAtTick: vi.fn() },
    },
    ui: {
      renderer: {
        getSelectionCoordinates: vi.fn().mockResolvedValue([100, 200]),
      },
    },
  },
}));

vi.mock("../services/popovers", async (importOriginal) => {
  const original =
    await importOriginal<typeof import("../services/popovers")>();
  return {
    ...original,
    parseBarlinePopover: vi.fn(),
    parseClefPopover: vi.fn(),
    parseKeySignaturePopover: vi.fn(),
    parseTimeSignaturePopover: vi.fn(),
  };
});

import {
  parseBarlinePopover,
  parseClefPopover,
  parseKeySignaturePopover,
  parseTimeSignaturePopover,
} from "../services/popovers";

// ---------------------------------------------------------------------------
// Fixture — RestSelection so we get tick directly without building a Track
// ---------------------------------------------------------------------------
const FLOW_ID = 5;
const STAVE_ID = 100;
const TICK = 64;

const stateWithSelection = createAppState({
  ui: {
    sequencer: {
      selection: {
        type: "Rest",
        flow_id: FLOW_ID,
        instrument_id: 10,
        stave_id: STAVE_ID,
        track_id: 300,
        tick: TICK,
      },
    },
  } as never,
});

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------
const setPopover = vi.fn();

function renderPopover(popover: PopoverState) {
  return renderWithProviders(
    <Popover popover={popover} setPopover={setPopover} />,
    { appState: stateWithSelection },
  );
}

describe("Popover", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // -------------------------------------------------------------------------
  // Visibility
  // -------------------------------------------------------------------------
  it("renders nothing when popover is null", () => {
    const { container } = renderPopover(null);
    expect(container.firstChild).toBeNull();
  });

  it("renders an input when popover is set", () => {
    renderPopover({ x: 100, y: 200, type: PopoverType.Barline });
    expect(screen.getByRole("textbox")).toBeInTheDocument();
  });

  it("positions the popover using inline style", () => {
    const { container } = renderPopover({
      x: 42,
      y: 99,
      type: PopoverType.Barline,
    });
    const root = container.firstChild as HTMLElement;
    expect(root.style.left).toBe("42px");
    expect(root.style.top).toBe("99px");
  });

  // -------------------------------------------------------------------------
  // Labels per type
  // -------------------------------------------------------------------------
  it("shows 'Barline' label for PopoverType.Barline", () => {
    renderPopover({ x: 0, y: 0, type: PopoverType.Barline });
    expect(screen.getByText("Barline")).toBeInTheDocument();
  });

  it("shows 'Clef' label for PopoverType.Clef", () => {
    renderPopover({ x: 0, y: 0, type: PopoverType.Clef });
    expect(screen.getByText("Clef")).toBeInTheDocument();
  });

  it("shows 'Key Signature' label for PopoverType.KeySignature", () => {
    renderPopover({ x: 0, y: 0, type: PopoverType.KeySignature });
    expect(screen.getByText("Key Signature")).toBeInTheDocument();
  });

  it("shows 'Time Signature' label for PopoverType.TimeSignature", () => {
    renderPopover({ x: 0, y: 0, type: PopoverType.TimeSignature });
    expect(screen.getByText("Time Signature")).toBeInTheDocument();
  });

  // -------------------------------------------------------------------------
  // Input value
  // -------------------------------------------------------------------------
  it("typing updates the input value", async () => {
    const user = userEvent.setup();
    renderPopover({ x: 0, y: 0, type: PopoverType.Barline });
    const input = screen.getByRole("textbox");
    await user.type(input, "||");
    expect(input).toHaveValue("||");
  });

  // -------------------------------------------------------------------------
  // Escape
  // -------------------------------------------------------------------------
  it("Escape calls setPopover(null) and clears value", async () => {
    const user = userEvent.setup();
    renderPopover({ x: 0, y: 0, type: PopoverType.Barline });
    const input = screen.getByRole("textbox");
    await user.type(input, "test");
    await user.keyboard("{Escape}");
    expect(setPopover).toHaveBeenCalledWith(null);
  });

  // -------------------------------------------------------------------------
  // Blur
  // -------------------------------------------------------------------------
  it("blur calls setPopover(null)", () => {
    renderPopover({ x: 0, y: 0, type: PopoverType.Barline });
    const input = screen.getByRole("textbox");
    fireEvent.blur(input);
    expect(setPopover).toHaveBeenCalledWith(null);
  });

  // -------------------------------------------------------------------------
  // Enter — valid dispatch per type
  // -------------------------------------------------------------------------
  it("Enter with valid barline input calls parseBarlinePopover and closes", async () => {
    const user = userEvent.setup();
    renderPopover({ x: 0, y: 0, type: PopoverType.Barline });
    await user.type(screen.getByRole("textbox"), "||{Enter}");
    expect(parseBarlinePopover).toHaveBeenCalledWith("||", FLOW_ID, TICK);
    expect(setPopover).toHaveBeenCalledWith(null);
  });

  it("Enter with valid clef input calls parseClefPopover and closes", async () => {
    const user = userEvent.setup();
    renderPopover({ x: 0, y: 0, type: PopoverType.Clef });
    await user.type(screen.getByRole("textbox"), "treble{Enter}");
    expect(parseClefPopover).toHaveBeenCalledWith(
      "treble",
      FLOW_ID,
      TICK,
      STAVE_ID,
    );
    expect(setPopover).toHaveBeenCalledWith(null);
  });

  it("Enter with valid key signature input calls parseKeySignaturePopover and closes", async () => {
    const user = userEvent.setup();
    renderPopover({ x: 0, y: 0, type: PopoverType.KeySignature });
    await user.type(screen.getByRole("textbox"), "G{Enter}");
    expect(parseKeySignaturePopover).toHaveBeenCalledWith("G", FLOW_ID, TICK);
    expect(setPopover).toHaveBeenCalledWith(null);
  });

  it("Enter with valid time signature input calls parseTimeSignaturePopover and closes", async () => {
    const user = userEvent.setup();
    renderPopover({ x: 0, y: 0, type: PopoverType.TimeSignature });
    await user.type(screen.getByRole("textbox"), "4/4{Enter}");
    expect(parseTimeSignaturePopover).toHaveBeenCalledWith(
      "4/4",
      FLOW_ID,
      TICK,
    );
    expect(setPopover).toHaveBeenCalledWith(null);
  });

  // -------------------------------------------------------------------------
  // Enter — invalid input should not close
  // -------------------------------------------------------------------------
  it("Enter with invalid input logs error and does NOT call setPopover", async () => {
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    vi.mocked(parseBarlinePopover).mockImplementationOnce(() => {
      throw new Error("Unknown input: xyz");
    });

    const user = userEvent.setup();
    renderPopover({ x: 0, y: 0, type: PopoverType.Barline });
    await user.type(screen.getByRole("textbox"), "xyz{Enter}");

    await waitFor(() => {
      expect(consoleError).toHaveBeenCalled();
    });
    expect(setPopover).not.toHaveBeenCalled();

    consoleError.mockRestore();
  });

  // -------------------------------------------------------------------------
  // Enter with empty value — no-op
  // -------------------------------------------------------------------------
  it("Enter with empty input does nothing", async () => {
    const user = userEvent.setup();
    renderPopover({ x: 0, y: 0, type: PopoverType.Barline });
    await user.keyboard("{Enter}");
    expect(parseBarlinePopover).not.toHaveBeenCalled();
    expect(setPopover).not.toHaveBeenCalled();
  });

  // -------------------------------------------------------------------------
  // Clef — null stave_id should not close or dispatch
  // -------------------------------------------------------------------------
  it("Enter for Clef does nothing when stave_id is null (no selection)", async () => {
    const user = userEvent.setup();
    const noSelectionState = createAppState();
    renderWithProviders(
      <Popover
        popover={{ x: 0, y: 0, type: PopoverType.Clef }}
        setPopover={setPopover}
      />,
      { appState: noSelectionState },
    );
    await user.type(screen.getByRole("textbox"), "treble{Enter}");
    expect(parseClefPopover).not.toHaveBeenCalled();
    expect(setPopover).not.toHaveBeenCalled();
  });

  // -------------------------------------------------------------------------
  // Tick from Tone selection
  // -------------------------------------------------------------------------
  it("Enter for Barline reads tick from a Tone selection entry", async () => {
    const user = userEvent.setup();
    const TRACK_ID = 500;
    const TONE_ID = 600;
    const TONE_TICK = 192;

    const stateWithTone = createAppState({
      score: {
        tracks: {
          by_id: {
            [TRACK_ID]: {
              id: TRACK_ID,
              entries: {
                by_tick: {},
                by_id: {
                  [TONE_ID]: {
                    id: TONE_ID,
                    type: "Tone",
                    tick: TONE_TICK,
                    duration: 128,
                    pitch: { step: 0, octave: 4, alter: 0 },
                    velocity: { value: 64 },
                    articulations: [],
                  },
                },
              },
            },
          },
        } as never,
      } as never,
      ui: {
        sequencer: {
          selection: {
            type: "Tone",
            flow_id: FLOW_ID,
            instrument_id: 10,
            stave_id: STAVE_ID,
            track_id: TRACK_ID,
            tone_id: TONE_ID,
          },
        },
      } as never,
    });

    renderWithProviders(
      <Popover
        popover={{ x: 0, y: 0, type: PopoverType.Barline }}
        setPopover={setPopover}
      />,
      { appState: stateWithTone },
    );

    await user.type(screen.getByRole("textbox"), "||{Enter}");
    expect(parseBarlinePopover).toHaveBeenCalledWith("||", FLOW_ID, TONE_TICK);
    expect(setPopover).toHaveBeenCalledWith(null);
  });

  // -------------------------------------------------------------------------
  // Keyboard shortcuts (document-level, Shift+key when popover is null)
  // -------------------------------------------------------------------------
  it("Shift+B fires setPopover with Barline type", async () => {
    renderPopover(null);
    fireEvent.keyDown(document, { key: "B", shiftKey: true });
    // async action — wait for setPopover
    await waitFor(() => {
      expect(setPopover).toHaveBeenCalledWith({
        x: 100,
        y: 200,
        type: PopoverType.Barline,
      });
    });
  });

  it("Shift+C fires setPopover with Clef type", async () => {
    renderPopover(null);
    fireEvent.keyDown(document, { key: "C", shiftKey: true });
    await waitFor(() => {
      expect(setPopover).toHaveBeenCalledWith({
        x: 100,
        y: 200,
        type: PopoverType.Clef,
      });
    });
  });

  it("Shift+K fires setPopover with KeySignature type", async () => {
    renderPopover(null);
    fireEvent.keyDown(document, { key: "K", shiftKey: true });
    await waitFor(() => {
      expect(setPopover).toHaveBeenCalledWith({
        x: 100,
        y: 200,
        type: PopoverType.KeySignature,
      });
    });
  });

  it("Shift+M fires setPopover with TimeSignature type", async () => {
    renderPopover(null);
    fireEvent.keyDown(document, { key: "M", shiftKey: true });
    await waitFor(() => {
      expect(setPopover).toHaveBeenCalledWith({
        x: 100,
        y: 200,
        type: PopoverType.TimeSignature,
      });
    });
  });

  it("keyboard shortcuts are skipped when popover is already open", async () => {
    // When popover is not null, the handler returns early
    renderPopover({ x: 0, y: 0, type: PopoverType.Barline });
    vi.clearAllMocks();
    fireEvent.keyDown(document, { key: "B", shiftKey: true });
    await new Promise((r) => setTimeout(r, 50));
    expect(setPopover).not.toHaveBeenCalled();
  });

  it("does not open popover when getSelectionCoordinates returns null", async () => {
    const { actions: mockActions } = await import("../services/actions");
    vi.mocked(
      mockActions.ui.renderer.getSelectionCoordinates,
    ).mockResolvedValueOnce(null);
    renderPopover(null);
    fireEvent.keyDown(document, { key: "B", shiftKey: true });
    await new Promise((r) => setTimeout(r, 50));
    expect(setPopover).not.toHaveBeenCalled();
  });
});
