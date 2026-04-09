import { describe, expect, it, vi, beforeEach } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { PlayingTechniqueKind } from "../../../../../services/types";
import { PlayingTechniquePropertiesSection } from "./playing-technique-properties-section";

vi.mock("../../../../../services/actions", () => ({
  actions: {
    score: {
      playingTechnique: {
        update: vi.fn().mockResolvedValue(undefined),
      },
    },
  },
}));

import { actions } from "../../../../../services/actions";

describe("PlayingTechniquePropertiesSection", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("dispatches the bounded hidden update when the hidden switch is toggled", () => {
    render(
      <PlayingTechniquePropertiesSection
        flowId={5}
        staveId={7}
        playingTechnique={{
          id: 9,
          tick: 64,
          type: "PlayingTechnique",
          hidden: false,
          kind: PlayingTechniqueKind.Pizz,
        }}
      />,
    );

    fireEvent.click(screen.getByRole("switch", { name: "Hidden" }));

    expect(actions.score.playingTechnique.update).toHaveBeenCalledWith(
      5,
      7,
      9,
      true,
    );
  });
});
