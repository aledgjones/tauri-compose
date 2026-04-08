import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Mixer, MixerToolbar } from "./mixer";
import {
  Instrument,
  InstrumentType,
  ResolvedPlaybackRoute,
  ResolvedPlaybackRouteStatus,
} from "../../../services/types";

vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      instrument: {
        setSolo: vi.fn(),
        setMute: vi.fn(),
        setPan: vi.fn(),
        setVolume: vi.fn(),
      },
    },
  },
}));

import { actions } from "../../../services/actions";

const makeInstrument = (overrides: Partial<Instrument> = {}): Instrument => ({
  id: 1,
  definition_id: "piano",
  instrument_type: InstrumentType.Melodic,
  long_name: "Piano",
  short_name: "Pno.",
  stave_ids: [],
  count: null,
  volume: 100,
  pan: 64,
  mute: false,
  solo: false,
  ...overrides,
});

const makeRoute = (
  overrides: Partial<ResolvedPlaybackRoute> = {},
): ResolvedPlaybackRoute => ({
  status: ResolvedPlaybackRouteStatus.Ready,
  slot_id: 1,
  slot_target: { plugin_id: "vst3:grand-piano" },
  effective_plugin_id: "vst3:grand-piano",
  target: { mode: 0, index: null },
  address: { mode: 0, index: null },
  supported_mix_controls: {
    volume: true,
    pan: true,
    mute: true,
    solo: true,
  },
  ...overrides,
});

describe("MixerToolbar", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing when instruments list is empty", () => {
    const { container } = render(
      <MixerToolbar
        instruments={[]}
        resolvedPlaybackRouteByInstrumentId={{}}
      />,
    );
    expect(container.firstChild).toBeInTheDocument();
  });

  it("renders both solo and mute buttons for supported routes", () => {
    render(
      <MixerToolbar
        instruments={[makeInstrument()]}
        resolvedPlaybackRouteByInstrumentId={{ 1: makeRoute() }}
      />,
    );
    expect(
      screen.getByRole("button", { name: "Solo all supported instruments" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Mute all supported instruments" }),
    ).toBeInTheDocument();
  });

  it("exposes the active toolbar toggle state without relying on color alone", () => {
    render(
      <MixerToolbar
        instruments={[makeInstrument({ solo: true, mute: false })]}
        resolvedPlaybackRouteByInstrumentId={{ 1: makeRoute() }}
      />,
    );

    expect(
      screen.getByRole("button", { name: "Solo all supported instruments" }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      screen.getByRole("button", { name: "Mute all supported instruments" }),
    ).toHaveAttribute("aria-pressed", "false");
  });

  it("calls setSolo only for instruments whose active route supports solo", () => {
    const instruments = [
      makeInstrument({ id: 1, solo: false }),
      makeInstrument({ id: 2, solo: false }),
    ];
    render(
      <MixerToolbar
        instruments={instruments}
        resolvedPlaybackRouteByInstrumentId={{
          1: makeRoute(),
          2: makeRoute({
            supported_mix_controls: {
              volume: false,
              pan: false,
              mute: false,
              solo: false,
            },
          }),
        }}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Solo all supported instruments" }),
    );
    expect(actions.score.instrument.setSolo).toHaveBeenCalledWith(1, true);
    expect(actions.score.instrument.setSolo).not.toHaveBeenCalledWith(2, true);
  });

  it("calls setMute only for instruments whose active route supports mute", () => {
    const instruments = [
      makeInstrument({ id: 1, mute: false }),
      makeInstrument({ id: 2, mute: false }),
    ];
    render(
      <MixerToolbar
        instruments={instruments}
        resolvedPlaybackRouteByInstrumentId={{
          1: makeRoute(),
          2: makeRoute({
            supported_mix_controls: {
              volume: false,
              pan: false,
              mute: false,
              solo: false,
            },
          }),
        }}
      />,
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Mute all supported instruments" }),
    );
    expect(actions.score.instrument.setMute).toHaveBeenCalledWith(1, true);
    expect(actions.score.instrument.setMute).not.toHaveBeenCalledWith(2, true);
    expect(actions.score.instrument.setSolo).not.toHaveBeenCalled();
  });
});

describe("Mixer", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing when no instruments", () => {
    const { container } = render(
      <Mixer instruments={[]} resolvedPlaybackRouteByInstrumentId={{}} />,
    );
    expect(container.firstChild).toBeInTheDocument();
  });

  it("renders a MixerChannel for each instrument", () => {
    const instruments = [
      makeInstrument({ id: 1, short_name: "Vln." }),
      makeInstrument({ id: 2, short_name: "Vc." }),
    ];
    render(
      <Mixer
        instruments={instruments}
        resolvedPlaybackRouteByInstrumentId={{
          1: makeRoute(),
          2: makeRoute(),
        }}
      />,
    );
    expect(screen.getByText("Vln.")).toBeInTheDocument();
    expect(screen.getByText("Vc.")).toBeInTheDocument();
  });

  it("wires the pan slider to score instrument pan updates", () => {
    render(
      <Mixer
        instruments={[makeInstrument({ id: 1, short_name: "Vln.", pan: 20 })]}
        resolvedPlaybackRouteByInstrumentId={{ 1: makeRoute() }}
      />,
    );

    const panControl = screen.getByRole("slider", { name: /Pan Vln\./i });
    vi.spyOn(panControl, "getBoundingClientRect").mockReturnValue({
      x: 0,
      y: 0,
      width: 100,
      height: 36,
      top: 0,
      right: 100,
      bottom: 36,
      left: 0,
      toJSON: () => ({}),
    });

    fireEvent.pointerDown(panControl, { clientX: 75.6 });

    expect(actions.score.instrument.setPan).toHaveBeenCalledWith(1, 96);
  });
});
