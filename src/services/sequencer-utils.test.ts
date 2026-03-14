import { describe, it, expect, beforeEach } from "vitest";
import { getSequencerClickTarget } from "./sequencer-utils";

describe("getSequencerClickTarget", () => {
  let container: HTMLDivElement;

  beforeEach(() => {
    container = document.createElement("div");
    document.body.appendChild(container);
  });

  it("returns null when element has no data-tone-id ancestor", () => {
    const el = document.createElement("span");
    container.appendChild(el);
    expect(getSequencerClickTarget(el)).toBeNull();
  });

  it("returns correct target when element IS the data-tone-id element", () => {
    const el = document.createElement("div");
    el.setAttribute("data-tone-id", "42");
    container.appendChild(el);

    const result = getSequencerClickTarget(el);
    expect(result).toEqual({ type: "Tone", id: 42 });
  });

  it("returns correct target when element is a CHILD of a data-tone-id element", () => {
    const parent = document.createElement("div");
    parent.setAttribute("data-tone-id", "7");
    const child = document.createElement("span");
    parent.appendChild(child);
    container.appendChild(parent);

    const result = getSequencerClickTarget(child);
    expect(result).toEqual({ type: "Tone", id: 7 });
  });

  it("returns a tuplet target when element is inside a data-tuplet-id region", () => {
    const parent = document.createElement("div");
    parent.setAttribute("data-tuplet-id", "17");
    const child = document.createElement("span");
    parent.appendChild(child);
    container.appendChild(parent);

    const result = getSequencerClickTarget(child);
    expect(result).toEqual({ type: "Tuplet", id: 17 });
  });

  it("returns correct target for deeply nested child", () => {
    const ancestor = document.createElement("div");
    ancestor.setAttribute("data-tone-id", "99");
    const mid = document.createElement("div");
    const inner = document.createElement("span");
    mid.appendChild(inner);
    ancestor.appendChild(mid);
    container.appendChild(ancestor);

    const result = getSequencerClickTarget(inner);
    expect(result).toEqual({ type: "Tone", id: 99 });
  });

  it("id is parsed as a number", () => {
    const el = document.createElement("div");
    el.setAttribute("data-tone-id", "123");
    container.appendChild(el);

    const result = getSequencerClickTarget(el);
    expect(typeof result?.id).toBe("number");
    expect(result?.id).toBe(123);
  });
});
