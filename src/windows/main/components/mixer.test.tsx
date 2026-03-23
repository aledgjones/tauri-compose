import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Mixer, MixerToolbar } from "./mixer";
import { Instrument, InstrumentType } from "../../../services/types";

vi.mock("../../../services/actions", () => ({
  actions: {
    score: {
      instrument: {
        setSolo: vi.fn(),
        setMute: vi.fn(),
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

describe("MixerToolbar", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing when instruments list is empty", () => {
    render(<MixerToolbar instruments={[]} />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(2);
  });

  it("renders S and M buttons", () => {
    render(<MixerToolbar instruments={[makeInstrument()]} />);
    const buttons = screen.getAllByRole("button");
    expect(buttons.length).toBeGreaterThanOrEqual(2);
  });

  it("calls setSolo for all instruments when S button clicked (none soloed → all soloed)", () => {
    const instruments = [
      makeInstrument({ id: 1, solo: false }),
      makeInstrument({ id: 2, solo: false }),
    ];
    render(<MixerToolbar instruments={instruments} />);
    const [soloBtn] = screen.getAllByRole("button");
    fireEvent.click(soloBtn);
    expect(actions.score.instrument.setSolo).toHaveBeenCalledWith(1, true);
    expect(actions.score.instrument.setSolo).toHaveBeenCalledWith(2, true);
  });

  it("calls setSolo(false) for all instruments when S clicked and some are soloed", () => {
    const instruments = [
      makeInstrument({ id: 1, solo: true }),
      makeInstrument({ id: 2, solo: false }),
    ];
    render(<MixerToolbar instruments={instruments} />);
    const [soloBtn] = screen.getAllByRole("button");
    fireEvent.click(soloBtn);
    expect(actions.score.instrument.setSolo).toHaveBeenCalledWith(1, false);
    expect(actions.score.instrument.setSolo).toHaveBeenCalledWith(2, false);
  });

  it("calls setMute for all instruments when M button clicked", () => {
    const instruments = [
      makeInstrument({ id: 1, mute: false }),
      makeInstrument({ id: 2, mute: false }),
    ];
    render(<MixerToolbar instruments={instruments} />);
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[1]); // mute button is 2nd
    expect(actions.score.instrument.setMute).toHaveBeenCalledWith(1, true);
    expect(actions.score.instrument.setMute).toHaveBeenCalledWith(2, true);
  });
});

describe("Mixer", () => {
  beforeEach(() => vi.clearAllMocks());

  it("renders without crashing when no instruments", () => {
    const { container } = render(<Mixer instruments={[]} />);
    expect(container.firstChild).toBeInTheDocument();
  });

  it("renders a MixerChannel for each instrument", () => {
    const instruments = [
      makeInstrument({ id: 1, short_name: "Vln." }),
      makeInstrument({ id: 2, short_name: "Vc." }),
    ];
    render(<Mixer instruments={instruments} />);
    // Each MixerChannel renders instrument.short_name via TokenizedText
    expect(screen.getByText("Vln.")).toBeInTheDocument();
    expect(screen.getByText("Vc.")).toBeInTheDocument();
  });
});
