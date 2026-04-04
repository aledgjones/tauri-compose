## MCP

When connected to atlassian-rovo-mcp:

- **MUST** use Jira project key = SC
- **MUST** use cloudId = "https://soloapps.atlassian.net" (do NOT call getAccessibleAtlassianResources)

## Coverage Analysis

- **MUST** use `npm run test:js:coverage:summary` to display coverage totals — do NOT write ad-hoc Python or Node scripts inline for this purpose
- **MUST** use `npm run test:js:coverage` to generate/refresh the coverage data before running `test:js:coverage:summary` if the data may be stale
- These scripts are safe to auto-approve; inline coverage scripts are not

## BMAD Bootstrap

- **MUST** invoke BMAD config loading through `./scripts/bmad-init` from the project root; do NOT call `python3 .github/skills/bmad-init/scripts/bmad_init.py` directly during normal agent work
- **MUST** treat `./scripts/bmad-init` as the repo-owned bootstrap for the BMAD local virtualenv so new agent sessions do not depend on shell-local activation state

## Visual Regression Cases

- **MUST NOT** add or restore committed visual-fixture corpora under `tests/fixtures/`
- **MUST** generate renderer-focused test cases on demand when manual visual inspection is needed, then discard those temporary artifacts before finishing the work
- **MUST** prefer seam-local automated tests over committed manual-regression assets whenever durable coverage is required
- **MUST** validate `.soloc` container changes through the relevant load/save tests and targeted local checks rather than a repo-owned fixture builder workflow

## Spec and Documentation Requirements

- **MUST** include documentation impact as part of every tech spec or implementation spec
- **MUST** identify which docs should be updated whenever a spec changes behavior, architecture, persistence, workflows, user-visible features, or developer expectations
- **MUST** include explicit documentation update tasks and acceptance criteria in specs; docs are not a follow-up nice-to-have
- **MUST** update AI-agent instructions as part of the work when project rules or required implementation practices change
- **MUST** treat score playback setup as rack-first: racks own slot lifecycle, slots own plugin assignment or unassignment, and instruments only own routes to already-existing slots; legacy direct instrument-to-plugin helpers are compatibility residue rather than the architecture to extend
- **MUST** treat the Story 1.3 single-instrument-VST-instance reset baseline and the follow-up Stories 1.4 through 1.6 as the gate that unlocked Epic 4 audible-playback work: automated backend trust checks plus maintainer manual proof were both required, and the completed gate still does not imply HALion-style multi-slot or other multi-timbral readiness, multi-instrument readiness, or already-shipped first-audible-playback breadth beyond the bounded Epic 4 starting point
- **MUST** delete or disconnect obsolete sampler-era, SCVI-era, or assignment-era product-path behavior once an approved VST-owned replacement seam exists; do not preserve removed playback paths through compatibility residue unless the active story explicitly requires a bounded temporary bridge
- **MUST NOT** satisfy playback stories by adding UI whose main purpose is to prove acceptance criteria literally when a simpler score-context control would satisfy the real product need; prefer the minimum useful surface that fits the rack-first model
- **MUST** preserve the tuplet timing invariant across parser, sequencer, and editing work: tuplet container geometry stays compressed to the denominator span on the host track, but crossing tone duration remains literal on the canonical expanded timeline and must not be compressed to the denominator span
- **MUST** treat tuplets as one shared semantic model across persistence, parsing, rendering, and playback; later consumers may apply that model for their own mechanics, but they must not re-derive ownership, boundary meaning, projection, or continuity through local subsystem-specific inference
- **MUST** keep tuplet regressions at the owning seam whenever local tests are sufficient; extend existing parser, renderer, playback, persistence, or editing tests before inventing a cross-cutting tuplet-only harness
- **MUST** treat shipped MVP tuplets as single-level only; nested `Tuplet` child entries remain intentionally unsupported until explicit post-MVP planning reopens that scope
- **MUST NOT** claim or implement broader arbitrary tuplet ratios or generalized sequencer-native tuplet editing UX unless the active story explicitly expands the shipped MVP scope
- **MUST** treat the current release-like `npm run perf:render` 3-instrument, 30-bar single-flow scenario as the active performance characterization baseline; trigger a regression warning when that representative pass reaches or exceeds 2ms total median time, and reevaluate the baseline and warning threshold as performance-sensitive features land
- **MUST** preserve `ui.flow_id` as the only authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport-facing consumers; do not introduce parallel frontend-owned active-flow state
- **MUST** keep the shared main-window top bar limited to universal controls; workflow-specific single-flow selectors belong inside the owning workflow surface such as Sequence
- **MUST** keep backend playback-route state and user-facing warning UI separate: backend seams may expose detailed non-ready states, but Sequence route controls stay minimal with plain `--` for unrouted instruments, `N. --` for empty selected slots, and warning icon/title treatment only for broken assigned plugin routes that actually need repair
- **MUST** treat supported `transport_play` as the backend-owned audible score path for the current MVP: reuse `prepare_supported_playback(...)`, start transport whenever `ui.flow_id` and the current tick are valid, snapshot supported routes once at play-start, and schedule score-derived events only for those supported routes instead of reverting to timing-only transport, audition-only playback, per-tick readiness probes, or any frontend-owned transport-audio model
- **MUST** keep transport clocking backend-owned and bounded: do not reintroduce one full wall-clock sleep per tick; preserve the short queue window, short wait quantum, and explicit lead-frame scheduling model so transport can absorb moderate scheduler jitter without moving timing ownership into the frontend or audio callback
- **MUST** treat edit-during-playback as a bounded backend-owned snapshot contract: only `volume`, `pan`, `mute`, and `solo` on already-ready direct-plugin root routes are immediate in the active pass; score-structure and playback-topology edits must persist canonical score intent immediately but defer audible effect until pause, seek restart, stopped play-start, natural-end replay, or another transport-owned reset boundary
- **MUST** treat pause, stop, and transport teardown as hard transport-owned resets over engine-owned transport work: clear queued transport events, retire active transport notes deterministically, leave no stale transport work armed after the thread joins, and make resume reuse the ordinary `transport_play` path from the paused tick with a fresh preparation snapshot
- **MUST** treat `prepare_supported_playback(...)` multi-target `supported_targets` as the canonical public transport snapshot; when one instrument-specific prepared target is needed, derive it through an explicit accessor such as `supported_target_for_instrument(...)` instead of restoring a public single-target compatibility field
- **MUST** keep the current MVP score-context mix controls honest: expose `volume`, `mute`, `solo`, and `pan` only for ready direct-plugin root routes proven through the backend-owned capability seam, and do not surface broader route-shape support until the same seam proves it end-to-end
- **MUST** push supported live mix edits into the actual engine mix seam used by in-flight playback, and **MUST** treat audio device or sample-rate stream rebuilds as transport-owned reset events that require a fresh post-rebuild play-start snapshot rather than preserving stale timing assumptions
- **MUST** keep shared written-duration parser test harness code under `src-tauri/src/instructions/parse/test_support/` and leave the musical scenario tables in `src-tauri/src/instructions/parse/get_written_durations.rs`; `get_written_durations_test_harness.rs` is the current home unless a focused follow-up intentionally splits that support by concern
- **MUST NOT** introduce a repo-wide notation test framework when parser-local test support is sufficient; low-level `get_written_durations.rs` helper tests should remain local direct unit tests
- **MUST** assess migration impact for any persisted-data change, `.soloc` payload/file-format change, or load/save semantic change; if older data would not load correctly without transformation, add or update the corresponding migration
- **MUST** author unreleased persisted-data migrations in `src-tauri/src/migrations/next.rs` until a human prepares the release
- **MUST** do persisted-data work in this order: implement the change, add/update pending migration steps in `src-tauri/src/migrations/next.rs`, then have a human run `prepare-release:*` and `finalize-release` last
- **MUST** keep the current tuplet migration stance explicit: legacy non-tuplet files do not need a dedicated tuplet migration today because pre-tuplet releases produced no tuplet-bearing payloads; add one only when a released older format truly requires transformation
- **MUST NOT** run release scripts or any release/tag/publish workflow; release commands are human-only
- **MUST** ensure shipped release states contain only concrete versioned migrations; `src-tauri/src/migrations/next.rs` is development-only and must be promoted/reseeded by the human prepare/finalize flow
- **MUST** update relevant docs whenever `.soloc` file-format behavior, persistence architecture, or migration rules change
- **MUST NOT** merge persisted-data changes that break older files without the required migration update and explicit migration-impact note

## Story And Review Discipline

- **MUST** continue to surface real bugs, missing requirements, and architectural risks during review, even when related future work exists
- **MUST** cross-reference planned future stories or explicitly deferred scope before classifying a concern as a defect in the current story; do not treat already-accounted-for future work as missing implementation in a stepping-stone story
- **MUST** distinguish between “current-story defect” and “valid concern owned by future work” in review output so scope is enforced without hiding real problems
- **MUST** ask clarifying questions before starting implementation when a story contains ambiguous or subjective decisions about behavior, UX, architecture, or quality expectations
- **MUST** prefer extending and refining existing APIs and seams over introducing parallel or duplicate flows unless the spec explicitly requires a new surface
