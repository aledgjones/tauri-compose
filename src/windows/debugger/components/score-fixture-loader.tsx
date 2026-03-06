import { mdiChevronDown, mdiChevronRight } from "@mdi/js";
import MdiIcon from "@mdi/react";
import { useMemo, useState } from "react";
import { actions } from "../../../services/actions";
import { Select } from "../../../ui/select";
import { scoreFixtures } from "../score-fixtures";

export const ScoreFixtureLoader = () => {
  const [selectedId, setSelectedId] = useState(scoreFixtures[0]?.id ?? "");
  const [isLoading, setIsLoading] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [showCases, setShowCases] = useState(false);

  const selectedFixture = useMemo(
    () => scoreFixtures.find((fixture) => fixture.id === selectedId) ?? null,
    [selectedId],
  );

  const fixtureOptions = useMemo(
    () =>
      scoreFixtures.map((fixture) => ({
        value: fixture.id,
        label: fixture.name,
      })),
    [],
  );

  const handleLoad = async () => {
    if (!selectedFixture || isLoading) {
      return;
    }

    setIsLoading(true);
    setStatus(null);

    try {
      await actions.debug.loadScoreFixture(selectedFixture.payload);
      setStatus(`Loaded ${selectedFixture.name}.`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setStatus(`Failed to load fixture: ${message}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="bg-neutral-900 p-3 mx-2 text-neutral-100 space-y-3">
      <div>
        <div className="text-sm font-semibold">Score fixtures</div>
        <div className="text-xs text-neutral-400 mt-1">
          Load a curated scenario directly into the running app. This replaces
          the current unsaved score state.
        </div>
      </div>

      <div className="flex items-end gap-2">
        <Select
          label="Fixture"
          value={selectedId || null}
          options={fixtureOptions}
          onChange={setSelectedId}
          variant="filled"
          className="flex-1"
        />

        <button
          type="button"
          onClick={handleLoad}
          disabled={isLoading || !selectedFixture}
          className="h-9 px-3 text-left text-white text-sm bg-neutral-800 hover:bg-neutral-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {isLoading ? "Loading…" : "Load"}
        </button>
      </div>

      {selectedFixture && (
        <>
          <div className="text-xs text-neutral-300">
            {selectedFixture.description}
          </div>

          {status && <div className="text-xs text-neutral-400">{status}</div>}

          <div>
            <button
              type="button"
              className="w-full flex items-center gap-2 text-xs font-semibold text-neutral-300"
              onClick={() => setShowCases((current) => !current)}
              aria-expanded={showCases}
            >
              <MdiIcon
                size={0.8}
                path={showCases ? mdiChevronDown : mdiChevronRight}
              />
              <span>Included cases</span>
            </button>

            {showCases && (
              <ul className="space-y-1 text-xs text-neutral-400 max-h-40 overflow-auto mt-2">
                {selectedFixture.cases.map((fixtureCase) => (
                  <li key={`${fixtureCase.measure}-${fixtureCase.name}`}>
                    {fixtureCase.flowTitle} · M{fixtureCase.measure} (
                    {fixtureCase.timeSignature}) — {fixtureCase.name}
                  </li>
                ))}
              </ul>
            )}
          </div>
        </>
      )}
    </div>
  );
};
