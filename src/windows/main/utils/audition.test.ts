import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { createAppState } from "../../../services/test-utils";
import { ResolvedPlaybackRouteStatus } from "../../../services/types";
import { auditionInstrumentPitch, canAuditionInstrument } from "./audition";

vi.mock("../../../services/actions", () => ({
  actions: {
    transport: {
      auditionTone: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

import { actions } from "../../../services/actions";

describe("audition utils", () => {
  let consoleError: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    vi.clearAllMocks();
    consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
  });

  afterEach(() => {
    consoleError.mockRestore();
  });

  it("requires the backend transport-ready flag before auditioning", () => {
    const state = createAppState({
      single_instrument_transport_ready: false,
      preferences: { audition: true },
      resolved_playback_route_by_instrument_id: {
        10: {
          status: ResolvedPlaybackRouteStatus.Ready,
          slot_id: 77,
          slot_target: { plugin_id: "plugin-abc" },
          effective_plugin_id: "plugin-abc",
          target: null,
          address: null,
        },
      },
    });

    expect(canAuditionInstrument(state, 10)).toBe(false);
  });

  it("logs and swallows audition failures", async () => {
    const state = createAppState({
      single_instrument_transport_ready: true,
      preferences: { audition: true },
      resolved_playback_route_by_instrument_id: {
        10: {
          status: ResolvedPlaybackRouteStatus.Ready,
          slot_id: 77,
          slot_target: { plugin_id: "plugin-abc" },
          effective_plugin_id: "plugin-abc",
          target: null,
          address: null,
        },
      },
    });
    const error = new Error("audition failed");
    vi.mocked(actions.transport.auditionTone).mockRejectedValueOnce(error);

    await expect(auditionInstrumentPitch(state, 10, 64)).resolves.toBe(false);
    expect(consoleError).toHaveBeenCalledWith("Failed to audition tone", error);
  });
});
