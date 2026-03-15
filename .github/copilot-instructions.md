## MCP

When connected to atlassian-rovo-mcp:

- **MUST** use Jira project key = SC
- **MUST** use cloudId = "https://soloapps.atlassian.net" (do NOT call getAccessibleAtlassianResources)

## Coverage Analysis

- **MUST** use `npm run test:js:coverage:summary` to display coverage totals — do NOT write ad-hoc Python or Node scripts inline for this purpose
- **MUST** use `npm run test:js:coverage` to generate/refresh the coverage data before running `test:js:coverage:summary` if the data may be stale
- These scripts are safe to auto-approve; inline coverage scripts are not

## Visual Regression Fixtures

- Fixture source files live in `tests/fixtures/source/` — plain JSON, committed to the repo
- Built `.soloc` files live in `tests/fixtures/built/` — gitignored, regenerated on demand
- **MUST** run `npm run build:fixtures` after adding or modifying any fixture JSON
- **MUST** group fixture files by concern (e.g. `time-signatures/`, `rhythms/`, `clefs/`, `key-signatures/`)
- **MUST** use flows to isolate individual test cases within a fixture file
- **MUST NOT** commit `.soloc` files — they are binary output and are gitignored
- See [tests/fixtures/README.md](../tests/fixtures/README.md) for the authoritative fixture guide
- Fixture JSON `score.meta.app_version` **MUST** match the current `CARGO_PKG_VERSION` (currently `1.23.3`) — the build script will fail with a clear error if it is missing
- When the `.soloc` container format changes, fixtures must be regenerated with `npm run build:fixtures`
- Compound fixture coverage is now valid where the app supports it; add representative variants under `tests/fixtures/source/rhythms/` when exercising compound notation behavior
- **MUST NOT** use Complex (5/4, 7/8, …) time signatures in fixtures until the renderer and load path support them end to end

## Spec and Documentation Requirements

- **MUST** include documentation impact as part of every tech spec or implementation spec
- **MUST** identify which docs should be updated whenever a spec changes behavior, architecture, persistence, workflows, user-visible features, or developer expectations
- **MUST** include explicit documentation update tasks and acceptance criteria in specs; docs are not a follow-up nice-to-have
- **MUST** update AI-agent instructions as part of the work when project rules or required implementation practices change
- **MUST** treat the current release-like `npm run perf:render` 3-instrument, 30-bar single-flow scenario as the active performance characterization baseline; trigger a regression warning when that representative pass reaches or exceeds 2ms total median time, and reevaluate the baseline and warning threshold as performance-sensitive features land
- **MUST** preserve `ui.flow_id` as the only authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport-facing consumers; do not introduce parallel frontend-owned active-flow state
- **MUST** keep the shared main-window top bar limited to universal controls; workflow-specific single-flow selectors belong inside the owning workflow surface such as Sequence
- **MUST** keep shared written-duration parser test harness code under `src-tauri/src/instructions/parse/test_support/` and leave the musical scenario tables in `src-tauri/src/instructions/parse/get_written_durations.rs`; `get_written_durations_test_harness.rs` is the current home unless a focused follow-up intentionally splits that support by concern
- **MUST NOT** introduce a repo-wide notation test framework when parser-local test support is sufficient; low-level `get_written_durations.rs` helper tests should remain local direct unit tests
- **MUST** assess migration impact for any persisted-data change, `.soloc` payload/file-format change, or load/save semantic change; if older data would not load correctly without transformation, add or update the corresponding migration
- **MUST** author unreleased persisted-data migrations in `src-tauri/src/migrations/next.rs` until a human prepares the release
- **MUST** do persisted-data work in this order: implement the change, add/update pending migration steps in `src-tauri/src/migrations/next.rs`, then have a human run `prepare-release:*` and `finalize-release` last
- **MUST NOT** run release scripts or any release/tag/publish workflow; release commands are human-only
- **MUST** ensure shipped release states contain only concrete versioned migrations; `src-tauri/src/migrations/next.rs` is development-only and must be promoted/reseeded by the human prepare/finalize flow
- **MUST** update relevant docs whenever `.soloc` file-format behavior, persistence architecture, or migration rules change
- **MUST NOT** merge persisted-data changes that break older files without the required migration update and explicit migration-impact note
