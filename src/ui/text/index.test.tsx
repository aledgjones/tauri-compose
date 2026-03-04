import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { TokenizedText } from "./index";
import { TokenizationContext, TokenMap } from "./context";

// ---------------------------------------------------------------------------
// Helper — renders TokenizedText with a given symbol map.
// ---------------------------------------------------------------------------
function renderWithSymbols(text: string, symbols: TokenMap) {
  return render(
    <TokenizationContext.Provider value={symbols}>
      <TokenizedText>{text}</TokenizedText>
    </TokenizationContext.Provider>,
  );
}

describe("TokenizedText", () => {
  it("renders plain text unchanged", () => {
    renderWithSymbols("Hello world", new Map());
    expect(screen.getByText("Hello world")).toBeInTheDocument();
  });

  it("renders a known symbol token as a font-bravura span", () => {
    const symbols: TokenMap = new Map([
      ["${sharp}", { render: () => "\u{E262}", sym: true }],
    ]);
    const { container } = renderWithSymbols("${sharp}", symbols);
    const span = container.querySelector("span.font-bravura");
    expect(span).toBeInTheDocument();
    expect(span?.textContent).toBe("\u{E262}");
  });

  it("renders a mix of plain text and symbol tokens", () => {
    const symbols: TokenMap = new Map([
      ["${sharp}", { render: () => "#", sym: true }],
    ]);
    const { container } = renderWithSymbols("C ${sharp} D", symbols);
    // Plain text parts should appear
    expect(container.textContent).toContain("C ");
    expect(container.textContent).toContain(" D");
    // Symbol rendered in bravura span
    const span = container.querySelector("span.font-bravura");
    expect(span).toBeInTheDocument();
  });

  it("renders unknown tokens as plain text (not stripped)", () => {
    const { container } = renderWithSymbols("${unknown}", new Map());
    expect(container.textContent).toContain("${unknown}");
  });

  it("renders multiple tokens in one string", () => {
    const symbols: TokenMap = new Map([
      ["${flat}", { render: () => "b", sym: true }],
      ["${sharp}", { render: () => "#", sym: true }],
    ]);
    const { container } = renderWithSymbols("${flat}${sharp}", symbols);
    const spans = container.querySelectorAll("span.font-bravura");
    expect(spans).toHaveLength(2);
  });
});
