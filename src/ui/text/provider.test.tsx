import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { renderHook } from "@testing-library/react";
import React, { useContext } from "react";
import { TokenizationProvider } from "./provider";
import { TokenizationContext, TokenMap } from "./context";
import { createWrapper, createAppState } from "../../services/test-utils";
import { AppState } from "../../services/types";

// ---------------------------------------------------------------------------
// Helper component that reads from the context and exposes the token map
// via a data attribute so the tests can inspect it easily.
// ---------------------------------------------------------------------------
const TokenConsumer: React.FC = () => {
  const tokens = useContext(TokenizationContext);

  // Collect all token keys into a comma-separated string for easy assertion
  const keys = Array.from(tokens.keys()).join(",");
  return <div data-testid="token-keys">{keys}</div>;
};

function renderProvider(metaOverrides?: Record<string, string>) {
  const state = createAppState(
    metaOverrides
      ? ({ score: { meta: metaOverrides } } as unknown as Partial<AppState>)
      : undefined,
  );

  return render(
    <TokenizationProvider>
      <TokenConsumer />
    </TokenizationProvider>,
    {
      wrapper: createWrapper({ appState: state }),
    },
  );
}

/** Renders the provider + consumer via renderHook and returns the token map. */
function captureTokens(state = createAppState()): TokenMap {
  const AppWrapper = createWrapper({ appState: state });
  const Wrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => (
    <AppWrapper>
      <TokenizationProvider>{children}</TokenizationProvider>
    </AppWrapper>
  );
  const { result } = renderHook(() => useContext(TokenizationContext), {
    wrapper: Wrapper,
  });
  return result.current;
}

describe("TokenizationProvider", () => {
  it("renders children", () => {
    renderProvider();
    expect(screen.getByTestId("token-keys")).toBeInTheDocument();
  });

  it("provides the ${project-title} token", () => {
    renderProvider();
    const keys = screen.getByTestId("token-keys").textContent ?? "";
    expect(keys).toContain("${project-title}");
  });

  it("project-title render() returns the meta title value", () => {
    const state = createAppState();
    state.score.meta.title = "My Symphony";

    const tokens = captureTokens(state);
    const token = tokens.get("${project-title}");
    expect(token).toBeDefined();
    expect(token!.render()).toBe("My Symphony");
    expect(token!.sym).toBe(false);
  });

  it("provides static symbol tokens (${copy}, ${sharp}, ${flat}, …)", () => {
    renderProvider();
    const keys = screen.getByTestId("token-keys").textContent ?? "";
    expect(keys).toContain("${copy}");
    expect(keys).toContain("${sharp}");
    expect(keys).toContain("${natural}");
    expect(keys).toContain("${flat}");
    expect(keys).toContain("${double-sharp}");
    expect(keys).toContain("${double-flat}");
  });

  it("${sharp} sym flag is true", () => {
    const tokens = captureTokens();
    const token = tokens.get("${sharp}");
    expect(token).toBeDefined();
    expect(token!.sym).toBe(true);
    expect(token!.render()).toBe("\u{E262}");
  });

  it("${copy} renders the copyright character", () => {
    const token = captureTokens().get("${copy}");
    expect(token?.render()).toBe("\u{00A9}");
  });

  it("provides all user-defined meta tokens", () => {
    renderProvider();
    const keys = screen.getByTestId("token-keys").textContent ?? "";
    const expected = [
      "${project-title}",
      "${project-subtitle}",
      "${project-dedication}",
      "${project-composer}",
      "${project-arranger}",
      "${project-lyricist}",
      "${project-artist}",
      "${project-copyist}",
      "${project-publisher}",
      "${project-editor}",
      "${project-copyright}",
      "${project-work-number}",
      "${project-composition-year}",
      "${project-composer-dates}",
      "${project-other-information}",
    ];
    for (const key of expected) {
      expect(keys).toContain(key);
    }
  });

  it("all user-defined meta tokens render() return the corresponding meta field", () => {
    const state = createAppState();
    state.score.meta.subtitle = "Subtitle";
    state.score.meta.dedication = "Dedication";
    state.score.meta.composer = "Composer";
    state.score.meta.arranger = "Arranger";
    state.score.meta.lyricist = "Lyricist";
    state.score.meta.artist = "Artist";
    state.score.meta.copyist = "Copyist";
    state.score.meta.publisher = "Publisher";
    state.score.meta.editor = "Editor";
    state.score.meta.copyright = "Copyright";
    state.score.meta.work_number = "Op.1";
    state.score.meta.composition_year = "1990";
    state.score.meta.composer_dates = "1960–2020";
    state.score.meta.other_information = "Other info";

    const tokens = captureTokens(state);
    expect(tokens.get("${project-subtitle}")?.render()).toBe("Subtitle");
    expect(tokens.get("${project-dedication}")?.render()).toBe("Dedication");
    expect(tokens.get("${project-composer}")?.render()).toBe("Composer");
    expect(tokens.get("${project-arranger}")?.render()).toBe("Arranger");
    expect(tokens.get("${project-lyricist}")?.render()).toBe("Lyricist");
    expect(tokens.get("${project-artist}")?.render()).toBe("Artist");
    expect(tokens.get("${project-copyist}")?.render()).toBe("Copyist");
    expect(tokens.get("${project-publisher}")?.render()).toBe("Publisher");
    expect(tokens.get("${project-editor}")?.render()).toBe("Editor");
    expect(tokens.get("${project-copyright}")?.render()).toBe("Copyright");
    expect(tokens.get("${project-work-number}")?.render()).toBe("Op.1");
    expect(tokens.get("${project-composition-year}")?.render()).toBe("1990");
    expect(tokens.get("${project-composer-dates}")?.render()).toBe("1960–2020");
    expect(tokens.get("${project-other-information}")?.render()).toBe(
      "Other info",
    );
  });

  it("remaining static symbol tokens render correct unicode", () => {
    const tokens = captureTokens();
    expect(tokens.get("${natural}")?.render()).toBe("\u{E261}");
    expect(tokens.get("${flat}")?.render()).toBe("\u{E260}");
    expect(tokens.get("${double-sharp}")?.render()).toBe("\u{E263}");
    expect(tokens.get("${double-flat}")?.render()).toBe("\u{E264}");
    // Verify sym=false for copy and sym=true for accidentals
    expect(tokens.get("${natural}")?.sym).toBe(true);
    expect(tokens.get("${double-sharp}")?.sym).toBe(true);
    expect(tokens.get("${double-flat}")?.sym).toBe(true);
  });
});
