import { describe, expect, it, vi, beforeEach } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import {
  NoteDuration,
  TimeSignatureDrawType,
} from "../../../../../services/types";
import { TimeSignaturePropertiesSection } from "./time-signature-properties-section";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      timeSignature: {
        update: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

describe("TimeSignaturePropertiesSection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("maps 4/4 symbol-form on to CommonTime", async () => {
    render(
      <TimeSignaturePropertiesSection
        flowId={5}
        timeSignature={{
          id: 7,
          tick: 64,
          type: "TimeSignature",
          hidden: false,
          beats: 4,
          beat_type: NoteDuration.Quarter,
          draw_type: TimeSignatureDrawType.Regular,
        }}
      />,
    );

    fireEvent.click(screen.getByRole("switch", { name: "Common/Cut common" }));

    await waitFor(() => {
      expect(actions.score.timeSignature.update).toHaveBeenCalledWith(
        5,
        7,
        TimeSignatureDrawType.CommonTime,
        false,
      );
    });
  });

  it("maps 2/2 symbol-form on to SplitCommonTime", async () => {
    render(
      <TimeSignaturePropertiesSection
        flowId={5}
        timeSignature={{
          id: 7,
          tick: 64,
          type: "TimeSignature",
          hidden: false,
          beats: 2,
          beat_type: NoteDuration.Half,
          draw_type: TimeSignatureDrawType.Regular,
        }}
      />,
    );

    fireEvent.click(screen.getByRole("switch", { name: "Common/Cut common" }));

    await waitFor(() => {
      expect(actions.score.timeSignature.update).toHaveBeenCalledWith(
        5,
        7,
        TimeSignatureDrawType.SplitCommonTime,
        false,
      );
    });
  });

  it("disables the symbol-form control for unsupported signatures", () => {
    render(
      <TimeSignaturePropertiesSection
        flowId={5}
        timeSignature={{
          id: 7,
          tick: 64,
          type: "TimeSignature",
          hidden: false,
          beats: 3,
          beat_type: NoteDuration.Quarter,
          draw_type: TimeSignatureDrawType.Regular,
        }}
      />,
    );

    const control = screen.getByRole("switch", {
      name: "Common/Cut common",
    });
    expect(control).toBeDisabled();

    fireEvent.click(control);
    expect(actions.score.timeSignature.update).not.toHaveBeenCalled();
  });

  it("does not flip the local switch when the backend update fails", async () => {
    vi.mocked(actions.score.timeSignature.update).mockRejectedValueOnce(
      new Error("invalid draw type"),
    );

    render(
      <TimeSignaturePropertiesSection
        flowId={5}
        timeSignature={{
          id: 7,
          tick: 64,
          type: "TimeSignature",
          hidden: false,
          beats: 4,
          beat_type: NoteDuration.Quarter,
          draw_type: TimeSignatureDrawType.Regular,
        }}
      />,
    );

    const control = screen.getByRole("switch", {
      name: "Common/Cut common",
    });
    expect(control).toHaveAttribute("aria-checked", "false");

    fireEvent.click(control);

    await waitFor(() => {
      expect(actions.score.timeSignature.update).toHaveBeenCalledWith(
        5,
        7,
        TimeSignatureDrawType.CommonTime,
        false,
      );
    });

    expect(control).toHaveAttribute("aria-checked", "false");
  });
});
