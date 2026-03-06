import type { ScoreFixtureDefinition } from "./types";

const fixtureModules = import.meta.glob("../test-cases/*.json", {
  eager: true,
  import: "default",
}) as Record<string, ScoreFixtureDefinition>;

export const scoreFixtures = Object.values(fixtureModules).sort((left, right) =>
  left.name.localeCompare(right.name),
);

export type { ScoreFixtureDefinition, ScoreFixturePayload } from "./types";
