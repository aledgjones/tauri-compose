import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { Playback } from "./playback";
import { renderWithProviders } from "../../../services/test-utils";

vi.mock("../../../services/actions", () => ({
  actions: {
    audio: {
      setDevice: vi.fn().mockResolvedValue(undefined),
      refreshDevices: vi.fn().mockResolvedValue(undefined),
      setDeviceSampleRate: vi.fn().mockResolvedValue(undefined),
    },
  },
}));

import { actions } from "../../../services/actions";

const DEVICE_A = {
  id: "dev-a",
  name: "Built-in Output",
  sample_rates: [44100, 48000],
  is_default: true,
  default_sample_rate: 44100,
};
const DEVICE_B = {
  id: "dev-b",
  name: "USB Audio",
  sample_rates: [48000, 96000],
  is_default: false,
  default_sample_rate: 48000,
};

const audioWithDevices = {
  devices: [DEVICE_A, DEVICE_B],
  active_device_id: "dev-a",
  current_sample_rate: 44100,
};

describe("Playback", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the Device Setup heading", () => {
    renderWithProviders(<Playback />, { audioState: audioWithDevices });
    expect(screen.getByText("Device Setup")).toBeInTheDocument();
  });

  it("renders the Output Device label", () => {
    renderWithProviders(<Playback />, { audioState: audioWithDevices });
    expect(screen.getByText("Output Device")).toBeInTheDocument();
  });

  it("renders the Sample Rate label", () => {
    renderWithProviders(<Playback />, { audioState: audioWithDevices });
    expect(screen.getByText("Sample Rate")).toBeInTheDocument();
  });

  it("renders with empty devices list without crashing", () => {
    renderWithProviders(<Playback />);
    expect(screen.getByText("Device Setup")).toBeInTheDocument();
  });

  it("renders device name in output device select trigger", () => {
    renderWithProviders(<Playback />, { audioState: audioWithDevices });
    expect(screen.getByText("Built-in Output")).toBeInTheDocument();
  });

  it("renders no sample rate selection when active device is not found", () => {
    const audioNoMatch = {
      ...audioWithDevices,
      active_device_id: "nonexistent",
    };
    renderWithProviders(<Playback />, { audioState: audioNoMatch });
    expect(screen.getByText("Sample Rate")).toBeInTheDocument();
  });

  it("calls actions.audio.refreshDevices when refresh button is clicked", () => {
    renderWithProviders(<Playback />, { audioState: audioWithDevices });
    // Buttons: [output-device-trigger (0), refresh-IconButton (1), sample-rate-trigger (2)]
    const buttons = screen.getAllByRole("button");
    fireEvent.click(buttons[1]);
    expect(actions.audio.refreshDevices).toHaveBeenCalledOnce();
  });
});
