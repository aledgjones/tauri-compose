import { describe, it, expect, vi, beforeEach } from "vitest";
import { hitTest } from "./hit-test";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

import { invoke } from "@tauri-apps/api/core";

describe("hitTest", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls invoke with correct command and snake_case arguments", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null);
    await hitTest(10, 20, 5, 8, 1.5);
    expect(invoke).toHaveBeenCalledWith("renderer_hit_test", {
      x: 10,
      y: 20,
      pan_x: 5,
      pan_y: 8,
      zoom: 1.5,
    });
  });

  it("returns the resolved value from invoke", async () => {
    const selection = { type: "Tempo", flow_id: 1, tempo_id: 8, tick: 64 };
    vi.mocked(invoke).mockResolvedValueOnce(selection);
    const result = await hitTest(0, 0, 0, 0, 1);
    expect(result).toEqual(selection);
  });

  it("passes through time signature, key signature, and clef hit results", async () => {
    const results = [
      {
        type: "TimeSignature",
        flow_id: 1,
        time_signature_id: 8,
        tick: 64,
      },
      {
        type: "KeySignature",
        flow_id: 1,
        key_signature_id: 9,
        tick: 64,
      },
      {
        type: "Clef",
        flow_id: 1,
        stave_id: 2,
        clef_id: 10,
        tick: 64,
      },
    ];

    for (const selection of results) {
      vi.mocked(invoke).mockResolvedValueOnce(selection);
      const result = await hitTest(0, 0, 0, 0, 1);
      expect(result).toEqual(selection);
    }
  });

  it("returns null when invoke resolves null", async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null);
    const result = await hitTest(0, 0, 0, 0, 1);
    expect(result).toBeNull();
  });
});
