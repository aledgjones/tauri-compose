import React from "react";
import { render, screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  AppContext,
  AudioContext,
  ExpressionMapLibraryContext,
  PluginInventoryContext,
  TransportContext,
  initialAppState,
  initialAudioState,
  initialExpressionMapLibraryState,
  initialPluginInventoryState,
  initialTransportState,
} from "./state/context";
import {
  AppState,
  AudioState,
  ExpressionMapLibraryState,
  PluginInventoryState,
  TransportState,
} from "./types";

export { screen, within, waitFor, userEvent };

// ---------------------------------------------------------------------------
// Deep merge helper — handles nested objects without clobbering sibling keys.
// Arrays are replaced (not merged) to keep fixture semantics simple.
// ---------------------------------------------------------------------------
function deepMerge<T>(base: T, overrides: Partial<T>): T {
  const result = structuredClone(base) as Record<string, unknown>;
  for (const key of Object.keys(overrides as object) as (keyof T)[]) {
    const baseVal = result[key as string];
    const overVal = (overrides as Record<string, unknown>)[key as string];
    if (
      overVal !== null &&
      typeof overVal === "object" &&
      !Array.isArray(overVal) &&
      baseVal !== null &&
      typeof baseVal === "object" &&
      !Array.isArray(baseVal)
    ) {
      result[key as string] = deepMerge(
        baseVal as Record<string, unknown>,
        overVal as Record<string, unknown>,
      );
    } else {
      result[key as string] = overVal;
    }
  }
  return result as T;
}

// ---------------------------------------------------------------------------
// Factory — creates a fresh AppState merged with caller-supplied overrides.
// Uses structuredClone to prevent cross-test mutation.
// ---------------------------------------------------------------------------
export function createAppState(overrides?: Partial<AppState>): AppState {
  const base = structuredClone(initialAppState);
  if (!overrides) return base;
  return deepMerge(base, overrides);
}

// ---------------------------------------------------------------------------
// renderWithProviders — wraps ui in all 4 context providers.
// ---------------------------------------------------------------------------
export type RenderOptions = {
  appState?: Partial<AppState>;
  audioState?: Partial<AudioState>;
  expressionMapLibraryState?: Partial<ExpressionMapLibraryState>;
  pluginInventoryState?: Partial<PluginInventoryState>;
  transportState?: Partial<TransportState>;
};

export function renderWithProviders(
  ui: React.ReactElement,
  options?: RenderOptions,
) {
  const appState = options?.appState
    ? deepMerge(structuredClone(initialAppState), options.appState)
    : structuredClone(initialAppState);

  const audioState = options?.audioState
    ? { ...structuredClone(initialAudioState), ...options.audioState }
    : structuredClone(initialAudioState);

  const pluginInventoryState = options?.pluginInventoryState
    ? {
        ...structuredClone(initialPluginInventoryState),
        ...options.pluginInventoryState,
      }
    : structuredClone(initialPluginInventoryState);

  const expressionMapLibraryState = options?.expressionMapLibraryState
    ? {
        ...structuredClone(initialExpressionMapLibraryState),
        ...options.expressionMapLibraryState,
      }
    : structuredClone(initialExpressionMapLibraryState);

  const transportState = options?.transportState
    ? { ...structuredClone(initialTransportState), ...options.transportState }
    : structuredClone(initialTransportState);

  function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <AppContext.Provider value={appState}>
        <AudioContext.Provider value={audioState}>
          <PluginInventoryContext.Provider value={pluginInventoryState}>
            <ExpressionMapLibraryContext.Provider
              value={expressionMapLibraryState}
            >
              <TransportContext.Provider value={transportState}>
                {children}
              </TransportContext.Provider>
            </ExpressionMapLibraryContext.Provider>
          </PluginInventoryContext.Provider>
        </AudioContext.Provider>
      </AppContext.Provider>
    );
  }

  return render(ui, { wrapper: Wrapper });
}

// ---------------------------------------------------------------------------
// createWrapper — returns a React wrapper component for use with renderHook.
// ---------------------------------------------------------------------------
export function createWrapper(
  options?: RenderOptions,
): React.FC<{ children: React.ReactNode }> {
  const appState = options?.appState
    ? deepMerge(structuredClone(initialAppState), options.appState)
    : structuredClone(initialAppState);

  const audioState = options?.audioState
    ? { ...structuredClone(initialAudioState), ...options.audioState }
    : structuredClone(initialAudioState);

  const pluginInventoryState = options?.pluginInventoryState
    ? {
        ...structuredClone(initialPluginInventoryState),
        ...options.pluginInventoryState,
      }
    : structuredClone(initialPluginInventoryState);

  const expressionMapLibraryState = options?.expressionMapLibraryState
    ? {
        ...structuredClone(initialExpressionMapLibraryState),
        ...options.expressionMapLibraryState,
      }
    : structuredClone(initialExpressionMapLibraryState);

  const transportState = options?.transportState
    ? { ...structuredClone(initialTransportState), ...options.transportState }
    : structuredClone(initialTransportState);

  return function Wrapper({ children }) {
    return (
      <AppContext.Provider value={appState}>
        <AudioContext.Provider value={audioState}>
          <PluginInventoryContext.Provider value={pluginInventoryState}>
            <ExpressionMapLibraryContext.Provider
              value={expressionMapLibraryState}
            >
              <TransportContext.Provider value={transportState}>
                {children}
              </TransportContext.Provider>
            </ExpressionMapLibraryContext.Provider>
          </PluginInventoryContext.Provider>
        </AudioContext.Provider>
      </AppContext.Provider>
    );
  };
}
