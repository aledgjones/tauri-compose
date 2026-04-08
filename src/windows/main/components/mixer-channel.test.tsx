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
    pan: 64,
    volume: 100,
    isMuted: false,
    isSoloed: false,
    onPanChange: vi.fn(),
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

  it("does not render a static pan label when pan control is unsupported", () => {
    renderMixerChannel();
    expect(screen.queryByText("C")).not.toBeInTheDocument();
  });

  it("calls onPanChange when the pan slider moves", () => {
    const onPanChange = vi.fn();
    renderMixerChannel({ onPanChange, showPanControl: true, pan: 64 });

    const panControl = screen.getByRole("slider", { name: /Pan Pno\./i });
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

    fireEvent.pointerDown(panControl, { clientX: 10 });

    expect(onPanChange).toHaveBeenCalledWith(13);
  });

  it("resets pan to center when the control is double clicked", () => {
    const onPanChange = vi.fn();
    renderMixerChannel({ onPanChange, showPanControl: true, pan: 12 });

    fireEvent.doubleClick(screen.getByRole("slider", { name: /Pan Pno\./i }));

    expect(onPanChange).toHaveBeenCalledWith(64);
  });

  it("calls onToggleSolo when solo button is clicked", () => {
    const onToggleSolo = vi.fn();
    renderMixerChannel({ onToggleSolo, showSoloControl: true });
    const soloButton = screen.getByRole("button", { name: "Solo Pno." });
    expect(soloButton).toHaveAttribute("aria-pressed", "false");
    fireEvent.click(soloButton);
    expect(onToggleSolo).toHaveBeenCalledOnce();
  });

  it("calls onToggleMute when mute button is clicked", () => {
    const onToggleMute = vi.fn();
    renderMixerChannel({ onToggleMute, showMuteControl: true });
    const muteButton = screen.getByRole("button", { name: "Mute Pno." });
    expect(muteButton).toHaveAttribute("aria-pressed", "false");
    fireEvent.click(muteButton);
    expect(onToggleMute).toHaveBeenCalledOnce();
  });

  it("exposes pressed state for selected solo and mute controls", () => {
    renderMixerChannel({
      showSoloControl: true,
      showMuteControl: true,
      isSoloed: true,
      isMuted: true,
    });

    expect(screen.getByRole("button", { name: "Solo Pno." })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(screen.getByRole("button", { name: "Mute Pno." })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });
});
