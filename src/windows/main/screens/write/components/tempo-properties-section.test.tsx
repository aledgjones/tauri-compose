import { describe, expect, it, vi, beforeEach } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { NoteDuration } from "../../../../../services/types";
import { TempoPropertiesSection } from "./tempo-properties-section";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      tempo: {
        update: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

const tempo = {
  id: 7,
  tick: 64,
  type: "Tempo" as const,
  text: "Allegro",
  beat_type: NoteDuration.Quarter,
  dots: 0,
  bpm: 120,
  hidden: false,
  show_text: true,
  show_metronome: true,
  parenthesized: false,
};

describe("TempoPropertiesSection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("commits text edits on blur", () => {
    render(<TempoPropertiesSection flowId={5} tempo={tempo} />);

    const input = screen.getByPlaceholderText("Tempo text");
    fireEvent.change(input, { target: { value: "Andante" } });
    fireEvent.blur(input);

    expect(actions.score.tempo.update).toHaveBeenCalledWith(
      5,
      7,
      "Andante",
      NoteDuration.Quarter,
      0,
      120,
      false,
      true,
      true,
      false,
    );
  });

  it("toggles text visibility immediately", () => {
    render(<TempoPropertiesSection flowId={5} tempo={tempo} />);

    fireEvent.click(screen.getByRole("switch", { name: "Show text" }));

    expect(actions.score.tempo.update).toHaveBeenCalledWith(
      5,
      7,
      "Allegro",
      NoteDuration.Quarter,
      0,
      120,
      false,
      false,
      true,
      false,
    );
  });

  it("updates bpm immediately on blur", () => {
    render(<TempoPropertiesSection flowId={5} tempo={tempo} />);

    const bpmInput = screen.getByDisplayValue("120");
    fireEvent.change(bpmInput, { target: { value: "132" } });
    fireEvent.blur(bpmInput);

    expect(actions.score.tempo.update).toHaveBeenCalledWith(
      5,
      7,
      "Allegro",
      NoteDuration.Quarter,
      0,
      132,
      false,
      true,
      true,
      false,
    );
  });

  it("updates beat type immediately", () => {
    render(<TempoPropertiesSection flowId={5} tempo={tempo} />);

    fireEvent.click(screen.getByTitle("Half note"));

    expect(actions.score.tempo.update).toHaveBeenCalledWith(
      5,
      7,
      "Allegro",
      NoteDuration.Half,
      0,
      120,
      false,
      true,
      true,
      false,
    );
  });

  it("toggles metronome visibility and parenthesized state immediately", () => {
    render(<TempoPropertiesSection flowId={5} tempo={tempo} />);

    fireEvent.click(screen.getByRole("switch", { name: "Show metronome" }));
    fireEvent.click(screen.getByRole("switch", { name: "Parenthesized" }));

    expect(actions.score.tempo.update).toHaveBeenNthCalledWith(
      1,
      5,
      7,
      "Allegro",
      NoteDuration.Quarter,
      0,
      120,
      false,
      true,
      false,
      false,
    );
    expect(actions.score.tempo.update).toHaveBeenNthCalledWith(
      2,
      5,
      7,
      "Allegro",
      NoteDuration.Quarter,
      0,
      120,
      false,
      true,
      false,
      true,
    );
  });
});
