import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { render } from "@testing-library/react";
import { MixerChannel } from "./mixer-channel";
import { TokenizationContext } from "../../../ui/text/context";
import type { Instrument } from "../../../services/types";
import { InstrumentType } from "../../../services/types";

const instrument: Instrument = {
  id: 1,
  definition_id: "piano",
  instrument_type: InstrumentType.Melodic,
  long_name: "Piano",
  short_name: "Pno.",
  stave_ids: [],
  count: null,
  assigned_plugin_id: null,
  volume: 100,
  pan: 0,
  mute: false,
  solo: false,
};

function renderMixerChannel(
  props: Partial<React.ComponentProps<typeof MixerChannel>> = {},
) {
  const defaults = {
    instrument,
    volume: 100,
    isMuted: false,
    isSoloed: false,
    onVolumeChange: vi.fn(),
    onToggleMute: vi.fn(),
    onToggleSolo: vi.fn(),
  };
  return render(
    <TokenizationContext.Provider value={new Map()}>
      <MixerChannel {...defaults} {...props} />
    </TokenizationContext.Provider>,
  );
}

describe("MixerChannel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders without crashing", () => {
    const { container } = renderMixerChannel();
    expect(container.firstChild).not.toBeNull();
  });

  it("renders the instrument short name", () => {
    renderMixerChannel();
    expect(screen.getByText("Pno.")).toBeInTheDocument();
  });

  it("calls onToggleSolo when solo button is clicked", () => {
    const onToggleSolo = vi.fn();
    renderMixerChannel({ onToggleSolo });
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[0]);
    expect(onToggleSolo).toHaveBeenCalledOnce();
  });

  it("calls onToggleMute when mute button is clicked", () => {
    const onToggleMute = vi.fn();
    renderMixerChannel({ onToggleMute });
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[1]);
    expect(onToggleMute).toHaveBeenCalledOnce();
  });
});
