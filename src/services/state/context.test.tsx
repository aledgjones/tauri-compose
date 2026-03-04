import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook } from "@testing-library/react";
import {
  useAppState,
  useAudioState,
  useSamplerState,
  useTransportState,
  AppContext,
  AudioContext,
  SamplerContext,
  TransportContext,
  initialAppState,
  initialAudioState,
  initialSamplerState,
  initialTransportState,
} from "./context";

// ---------------------------------------------------------------------------
// Suppress React error boundary console noise for expected throws
// ---------------------------------------------------------------------------
let consoleError: ReturnType<typeof vi.spyOn>;

beforeEach(() => {
  consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
});

afterEach(() => {
  consoleError.mockRestore();
});

describe("useAppState", () => {
  it("throws with correct message when used outside AppContext.Provider", () => {
    expect(() => renderHook(() => useAppState())).toThrow(
      "useAppState must be used within an AppStateProvider",
    );
  });

  it("returns state when rendered inside AppContext.Provider", () => {
    const { result } = renderHook(() => useAppState(), {
      wrapper: ({ children }) => (
        <AppContext.Provider value={initialAppState}>
          {children}
        </AppContext.Provider>
      ),
    });
    expect(result.current).toEqual(initialAppState);
  });
});

describe("useAudioState", () => {
  it("throws with correct message when used outside AudioContext.Provider", () => {
    expect(() => renderHook(() => useAudioState())).toThrow(
      "useAudioState must be used within an AudioStateProvider",
    );
  });

  it("returns state when rendered inside AudioContext.Provider", () => {
    const { result } = renderHook(() => useAudioState(), {
      wrapper: ({ children }) => (
        <AudioContext.Provider value={initialAudioState}>
          {children}
        </AudioContext.Provider>
      ),
    });
    expect(result.current).toEqual(initialAudioState);
  });
});

describe("useSamplerState", () => {
  it("throws with correct message when used outside SamplerContext.Provider", () => {
    expect(() => renderHook(() => useSamplerState())).toThrow(
      "useSamplerState must be used within an AppStateProvider",
    );
  });

  it("returns state when rendered inside SamplerContext.Provider", () => {
    const { result } = renderHook(() => useSamplerState(), {
      wrapper: ({ children }) => (
        <SamplerContext.Provider value={initialSamplerState}>
          {children}
        </SamplerContext.Provider>
      ),
    });
    expect(result.current).toEqual(initialSamplerState);
  });
});

describe("useTransportState", () => {
  it("throws with correct message when used outside TransportContext.Provider", () => {
    expect(() => renderHook(() => useTransportState())).toThrow(
      "useTransportState must be used within an AppStateProvider",
    );
  });

  it("returns state when rendered inside TransportContext.Provider", () => {
    const { result } = renderHook(() => useTransportState(), {
      wrapper: ({ children }) => (
        <TransportContext.Provider value={initialTransportState}>
          {children}
        </TransportContext.Provider>
      ),
    });
    expect(result.current).toEqual(initialTransportState);
  });
});
