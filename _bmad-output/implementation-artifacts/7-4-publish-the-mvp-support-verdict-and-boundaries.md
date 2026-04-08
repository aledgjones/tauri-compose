# Story 7.4: Publish The MVP Support Verdict And Boundaries

Status: done

## Story

As a maintainer,
I want a clear support verdict and boundary statement for MVP playback,
so that the team knows what can be shipped, what is known-good, and what remains outside scope.

## Acceptance Criteria

1. Given the representative plugin, host, and transport validations have been completed, when the MVP support verdict is produced, then it states whether Compose is trustworthy enough to support real composition work for the validated subset, and it names the actual scope of that supported subset.
2. Given some behaviors or plugins may remain outside the validated baseline, when the support verdict is documented, then unsupported or unproven areas are called out explicitly, and they are not described in a way that implies broader support than the evidence justifies.
3. Given the release baseline affects future planning as well as release confidence, when the verdict is published, then it feeds back into planning, QA, and product guidance, and later compatibility expansion can build from the validated boundary rather than re-litigating MVP scope.

## Tasks / Subtasks

- [x] Task 1: Publish one explicit release verdict in the canonical Epic 7 baseline artifact (AC: 1, 2, 3)
  - [x] Reuse `docs/playback-support-baseline.md` as the only MVP support-verdict surface; do not create a parallel release-readiness matrix in `TESTING.md`, `docs/development-guide.md`, `README.md`, or `releases.md`.
  - [x] Add a top-level verdict statement that answers the actual release question: whether Compose is composition-trustworthy for the bounded validated subset, not whether some isolated seams passed in isolation.
  - [x] Name the supported subset explicitly by platform, plugin format, playback topology, scenario coverage, and concrete bundle posture.

- [x] Task 2: Resolve or explicitly preserve the bundle-level boundary without over-claiming support (AC: 1, 2)
  - [x] Reuse Story 7.2, Story 7.2a, and Story 7.3 evidence exactly as recorded; do not silently promote `BBC Symphony Orchestra.vst3` or `Originals - Intimate Grand Piano.vst3` from backend automation alone.
  - [x] If maintainer manual revalidation is performed for BBC and Originals after the Story 7.2a host-lifecycle fixes, capture the concrete repeated open/focus/close/reopen and routed-playback evidence in the canonical baseline before any promotion to `known-good`.
  - [x] If that manual proof is not available during Story 7.4, preserve both bundles as `discovered but unvalidated` and make the final verdict honest about that remaining release boundary.
  - [x] Keep `Reverb.vst3` and `ProEQ.vst3` classified honestly as bounded host-behavior control bundles unless new routed-playback evidence is added for them.

- [x] Task 3: Publish unsupported, unproven, and future-expansion boundaries in plain release language (AC: 2, 3)
  - [x] Restate the MVP scope explicitly: macOS only, VST3 only, rack-first routed playback, direct-plugin root routes, backend-owned transport, and the currently validated scenario matrix.
  - [x] Call out the unsupported or unproven areas explicitly: blocked bundles in the bounded host seam, discovered-but-unvalidated bundles, broader plugin families, AU, Windows, Linux, multi-timbral or multi-slot workflows, and any richer expression-map realism beyond the validated path.
  - [x] Add a concise “what later work builds from here” boundary so future compatibility expansion starts from the validated MVP edge instead of re-opening Epic 7’s scope debate.

- [x] Task 4: Feed the verdict into planning, QA, and maintainer guidance without creating drift (AC: 3)
  - [x] Update `TESTING.md` only if the final verdict changes the recommended validation workflow, command set, or maintainer proof expectations; otherwise keep it as a lightweight pointer to the canonical baseline.
  - [x] Update `docs/development-guide.md` only if the final verdict changes maintainer-facing support posture or release-readiness guidance materially.
  - [x] Update `docs/index.md` only if Story 7.4 adds a new durable documentation artifact; otherwise keep the baseline doc as the existing canonical entry.
  - [x] Keep sprint and retrospective follow-through explicit: when Story 7.4 and the Epic 7 retrospective are complete, close Epic 7 in `sprint-status.yaml` in the same pass to avoid tracking drift.

## Dev Notes

### Story Intent

- Story 7.4 is the publication and release-boundary synthesis pass for Epic 7.
- The job is not to build a new support model. The job is to turn the evidence already gathered in Stories 7.1 through 7.3 into one honest, release-credible verdict.
- The strongest failure mode here is over-claiming: treating seam-level success as universal plugin support or treating unresolved bundle-level evidence as already shippable.

### Previous Story Intelligence

- Story 7.1 created `docs/playback-support-baseline.md` as the canonical Epic 7 support artifact and defined the vocabulary: `known-good`, `blocked/unsupported for the bounded seam`, `discovered but unvalidated`, and `future compatibility expansion`.
- Story 7.2 validated discovery and bounded host behavior through the real owner seams, but maintainer evidence still showed intermittent hangs for `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3`.
- Story 7.2a fixed the concrete bounded host-lifecycle faults in `src-tauri/src/playback/plugin_editor_poc.rs`, but explicitly left BBC and Originals at `discovered but unvalidated` pending maintainer manual revalidation.
- Story 7.3 proved the backend-owned transport contract is composition-trustworthy for the bounded transport scenario matrix, but it did not provide bundle-specific maintainer proof for BBC or Originals.
- Epic 7 therefore ends with a split evidence posture: the backend transport seam is credible, while concrete real-library bundle promotion still depends on explicit host-plus-transport validation on those bundles.

### This Story Owns

- The final MVP playback support verdict for the currently validated subset.
- The explicit bundle-level and scope-level boundary statement that prevents support drift.
- Any bounded maintainer manual revalidation needed to promote BBC or Originals beyond `discovered but unvalidated`.
- Documentation updates that turn Epic 7 evidence into release, planning, and QA guidance.

### This Story Does Not Own

- New playback architecture, transport redesign, or host-lifecycle redesign.
- A second validation matrix, release dashboard, or frontend-owned support taxonomy.
- Blanket VST3 compatibility claims, broader plugin-family claims, or non-macOS support claims.
- Running release scripts, tagging, or publishing a release.

### Current Relevant Seams

- `docs/playback-support-baseline.md`
  - Canonical Epic 7 evidence, vocabulary, bundle posture, scenario baseline, and the correct place to publish the final verdict.
- `docs/development-guide.md`
  - Maintainer-facing setup and support-posture guidance that should summarize the verdict only if the workflow meaning changes materially.
- `TESTING.md`
  - QA and maintainer validation command guidance that should continue to point at the canonical baseline rather than duplicating its matrix.
- `docs/index.md`
  - Documentation index entry for the baseline artifact; only change it if Story 7.4 introduces a new durable doc.
- `_bmad-output/implementation-artifacts/7-1-define-the-supported-plugin-and-scenario-baseline.md`
  - Origin of the support vocabulary and the rule that the baseline document owns Epic 7 truth.
- `_bmad-output/implementation-artifacts/7-2-validate-plugin-discovery-and-host-behavior-against-the-baseline.md`
  - Discovery and bounded-host validation evidence and the original unresolved bundle verdict.
- `_bmad-output/implementation-artifacts/7-2a-stabilize-bounded-native-editor-host-for-real-library-control-bundles.md`
  - Structural repair story for the BBC and Originals host-lifecycle faults and the post-fix revalidation requirement.
- `_bmad-output/implementation-artifacts/7-3-validate-transport-trust-against-representative-composition-scenarios.md`
  - Backend-owned transport-trust verdict and the explicit caution against promoting BBC or Originals from automation alone.
- `src-tauri/src/playback/plugin_editor_poc.rs`
  - Owner seam for any host-lifecycle evidence cited in the final verdict.
- `src-tauri/src/playback/plugin_host_compatibility.rs`
  - Owner seam for the blocked-bundle list; Story 7.4 must not contradict it.
- `src-tauri/src/playback/plugin_inventory.rs`
  - Owner seam for discovery and availability truth used by `discovered but unvalidated` posture.
- `src-tauri/src/playback/preparation.rs`
  - Owner seam for `supported_targets` and routed-playback readiness evidence.
- `src-tauri/src/playback/transport.rs`
  - Owner seam for the bounded transport-trust verdict.

### Repo Memory Guardrails

- When an epic is actually complete and its retrospective is done, update epic-level sprint status in the same closing pass to avoid drift.
- Support publication must follow backend-owned truth; do not let release language outrun owner-seam evidence.
- Story 7.3’s transport verdict is strong enough to justify a bounded composition-trust claim at the seam level, but not strong enough to imply that every concrete bundle in the representative library set is now `known-good`.

### Implementation Guidance

- Start from `docs/playback-support-baseline.md` and keep the final verdict there. If the document needs better structure, refactor that doc rather than spinning up a second verdict artifact.
- Publish the verdict at three levels:
  - product-level: what Compose MVP playback can honestly claim today;
  - bundle-level: which concrete bundles are `known-good`, `blocked/unsupported for the bounded seam`, or `discovered but unvalidated`;
  - future-boundary level: what remains intentionally outside MVP.
- Use direct, release-style language. The verdict should answer “Can this ship, for whom, and under what constraints?” rather than restating the validation tables in softer prose.
- Keep the distinction between seam proof and bundle proof explicit:
  - backend transport trust can be composition-credible;
  - a real-library bundle can still remain unvalidated if host-plus-transport maintainer proof is missing.
- If Story 7.4 includes maintainer revalidation for BBC or Originals, capture the actual scenario coverage required for promotion:
  - repeated open/focus/close/reopen after the Story 7.2a fixes;
  - routed play start;
  - pause/resume;
  - stopped and active seek;
  - any bounded edit-during-playback scenario needed to support the release wording.
- If that proof cannot be captured, do not soften the caveat. A narrower honest verdict is correct; a broader unsupported one is not.
- Keep `releases.md` out of the evidence path unless a human later prepares an actual release. Story 7.4 is about support truth, not release-note authorship.

### Risks And Regression Traps

- Promoting `BBC Symphony Orchestra.vst3` or `Originals - Intimate Grand Piano.vst3` to `known-good` without post-fix maintainer revalidation.
- Collapsing `known-good`, `blocked/unsupported for the bounded seam`, and `discovered but unvalidated` into mushy “generally supported” language.
- Treating `Reverb.vst3` or `ProEQ.vst3` host-control evidence as routed score-playback evidence.
- Letting `TESTING.md`, `docs/development-guide.md`, or any future release notes become a second drifting support matrix.
- Turning the story into a new architecture or bug-fix effort instead of a bounded evidence-and-verdict publication pass.
- Closing Epic 7 tracking incompletely by marking the story done without also handling the retrospective and epic-level state.

### Testing Requirements

- Prefer evidence validation over new code changes. Story 7.4 is expected to be documentation synthesis plus bounded maintainer proof unless new defects are discovered.
- If no code changes are required, validate every published claim against the existing owner-seam evidence already recorded in:
  - `docs/playback-support-baseline.md`
  - Story 7.2 / 7.2a / 7.3 artifacts
  - owner seams under `src-tauri/src/playback/`
- If bundle promotion is attempted for BBC or Originals, manual validation must cover the post-fix host lifecycle plus the Story 7.3 routed transport matrix on those concrete bundles before support language changes.
- If implementation uncovers code defects and changes owner seams, rerun the relevant commands before publishing the final verdict:
  - `cd src-tauri && cargo test plugin_editor_poc --lib -- --nocapture`
  - `cd src-tauri && cargo test plugin_inventory --lib -- --nocapture`
  - `cd src-tauri && cargo test transport_ --lib -- --nocapture`
  - `cd src-tauri && cargo test prepare_supported_playback --lib -- --nocapture`
  - `cd src-tauri && cargo test expression_maps --lib -- --nocapture`
  - `cd src-tauri && cargo test playback::engine --lib -- --nocapture`
  - `npm run lint`
  - `npm run check-types`
  - `npm run test:js`
  - `npm run test:rust`

### Documentation Impact

- Update `docs/playback-support-baseline.md` with the final Story 7.4 verdict and any bundle-level promotion or retained caveat.
- Update `docs/development-guide.md` only if the final verdict changes maintainer-facing release or support guidance materially.
- Update `TESTING.md` only if the final verdict changes the recommended validation workflow materially.
- Update `docs/index.md` only if Story 7.4 adds a new durable doc.
- Update sprint artifacts and the Epic 7 retrospective as part of the closeout flow; the final verdict should feed back into planning rather than remaining trapped in one doc.
- Update AI instructions only if Story 7.4 establishes a durable repo-wide rule for support publication beyond Epic 7.

### Architecture Compliance

- Backend-owned truth remains canonical: support claims must follow Rust-owned discovery, host, preparation, and transport evidence.
- The MVP claim remains explicitly macOS-only, VST3-only, rack-first, and bounded to direct-plugin root-route playback on the current backend-owned transport path.
- Legacy sampler, SCVI, or frontend-owned fallback claims must not reappear to make the verdict sound broader.
- The final verdict must stay honest about the difference between validated scope and future compatibility expansion.

### Library And Framework Requirements

- Stay on the existing repo baselines: Rust 2021, Tauri 2.10.1, React 19.2.4, TypeScript 5.9.3.
- No new dependencies are expected for Story 7.4.

### File Structure Notes

- Keep the final support verdict in `docs/` alongside the existing canonical baseline.
- Keep story-tracking and implementation notes under `_bmad-output/implementation-artifacts/`.
- Do not create a standalone validation app, frontend support surface, or committed fixture corpus for this publication story.

### Git Intelligence Summary

- Recent Epic 7 work followed the correct order: baseline definition first (`feat(7.1): playback support docs.`), then bounded validation (`feat(7.2): validation`), then host-lifecycle repair (`feat(7.2a): natuve plugin ui fixes`), then transport trust validation (`feat(7.3): validation of vst routes`).
- The touched files reinforce the intended Story 7.4 pattern: update the canonical baseline, update surrounding guidance only when materially necessary, keep sprint artifacts aligned, and avoid widening implementation beyond the owner seams.

### Latest Technical Information

- No external dependency upgrade or fresh framework research is required for Story 7.4.
- The decisive “latest” information for this story is current repo truth as of 2026-04-08: the canonical support baseline, the completed Epic 7 validation artifacts, the current owner seams, and the still-open bundle-level revalidation boundary for BBC and Originals.

### References

- `_bmad-output/planning-artifacts/epics.md`
  - Epic 7 and Story 7.4 acceptance criteria.
- `_bmad-output/planning-artifacts/prd.md`
  - Journey 4 release-credibility framing, FR39, and FR40.
- `_bmad-output/planning-artifacts/architecture.md`
  - Backend-owned playback truth, release-trust framing, and score-context versus app-scoped boundaries.
- `_bmad-output/planning-artifacts/implementation-readiness-report-2026-03-20.md`
  - FR39, FR40, NFR17, and the reminder that Epic 7 is a release-hardening evidence track.
- `docs/playback-support-baseline.md`
  - Canonical Epic 7 evidence and the target publication surface for the final verdict.
- `docs/development-guide.md`
  - Maintainer-facing guidance that currently defers to the canonical baseline.
- `TESTING.md`
  - Validation commands and the rule that the baseline doc owns the Epic 7 matrix.
- `docs/index.md`
  - Current documentation index entry for the baseline artifact.
- `_bmad-output/implementation-artifacts/7-1-define-the-supported-plugin-and-scenario-baseline.md`
  - Support-vocabulary and canonical-artifact origin.
- `_bmad-output/implementation-artifacts/7-2-validate-plugin-discovery-and-host-behavior-against-the-baseline.md`
  - Discovery and bounded-host evidence.
- `_bmad-output/implementation-artifacts/7-2a-stabilize-bounded-native-editor-host-for-real-library-control-bundles.md`
  - Post-fix revalidation requirement for BBC and Originals.
- `_bmad-output/implementation-artifacts/7-3-validate-transport-trust-against-representative-composition-scenarios.md`
  - Backend-owned transport verdict and bundle-promotion caution.
- `/memories/repo/sprint-status-epic-close-discipline.md`
  - Epic closeout tracking discipline.

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Implementation Plan

- Publish the final Story 7.4 verdict only in `docs/playback-support-baseline.md` and keep surrounding docs as pointers unless workflow guidance materially changes.
- Reuse Story 7.2, 7.2a, and 7.3 evidence exactly as already recorded so BBC and Originals stay unpromoted without fresh maintainer proof.
- Move Story 7.4 to `review` after validation while leaving Epic 7 itself `in-progress` because the retrospective has not been completed in this pass.

### Debug Log References

- 2026-04-08: Story prepared from Epic 7 planning, PRD release-credibility requirements, architecture ownership rules, implementation-readiness findings, current support-baseline docs, and prior Epic 7 validation artifacts.
- 2026-04-08: Story context explicitly separates backend transport proof from bundle-level host-plus-transport proof so Story 7.4 does not accidentally over-claim support for BBC or Originals.
- 2026-04-08: Story context keeps `docs/playback-support-baseline.md` as the single publication surface for the final verdict and limits surrounding docs to summary guidance only when necessary.
- 2026-04-08: Published the explicit Story 7.4 release verdict, supported-subset statement, unsupported-boundary language, and planning guidance in `docs/playback-support-baseline.md` without creating a second support matrix.
- 2026-04-08: Reused the recorded Story 7.2, 7.2a, and 7.3 evidence exactly as written, preserving `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3` as `discovered but unvalidated` and keeping `Reverb.vst3` / `ProEQ.vst3` scoped to bounded host-behavior proof.
- 2026-04-08: Confirmed `TESTING.md`, `docs/development-guide.md`, and `docs/index.md` already point to the canonical baseline cleanly, so no secondary documentation changes were needed.

### Completion Notes List

- Story context created for Story 7.4.
- Story 7.4 moved to `ready-for-dev` in sprint tracking.
- The story directs implementation toward one explicit MVP support verdict, one honest supported-subset statement, and one explicit unsupported or unproven boundary statement.
- The story also carries forward Epic 7 closeout discipline so the retrospective and epic-level sprint state do not drift after the final story is completed.
- Published the final MVP verdict in `docs/playback-support-baseline.md`: Compose is composition-trustworthy for the bounded backend-owned macOS VST3 routed-playback seam, but not for a broader real-library or cross-platform support claim.
- Preserved `BBC Symphony Orchestra.vst3` and `Originals - Intimate Grand Piano.vst3` as `discovered but unvalidated` because no post-Story-7.2a maintainer manual revalidation evidence was added in this pass.
- Left `Reverb.vst3` and `ProEQ.vst3` classified as bounded host-behavior control bundles only, and kept blocked bundles and broader plugin or platform claims outside the MVP support verdict.
- Left `TESTING.md`, `docs/development-guide.md`, and `docs/index.md` unchanged because the final verdict did not change the validation workflow, maintainer setup guidance, or documentation inventory.
- Validation passed: `npm run lint`, `npm run check-types`, `npm run test:js`, and `npm run test:rust`.
- Story 7.4 moved to `review`; `epic-7` remains `in-progress` because `epic-7-retrospective` has not been completed, so epic closeout remains explicitly deferred.
- 2026-04-08: Maintainer accepted the Story 7.4 review pass as-is and requested final completion without further changes.

### File List

- `_bmad-output/implementation-artifacts/7-4-publish-the-mvp-support-verdict-and-boundaries.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `docs/playback-support-baseline.md`

### Change Log

- 2026-04-08: Created the Story 7.4 implementation artifact and advanced sprint tracking to `ready-for-dev`.
- 2026-04-08: Published the bounded MVP playback support verdict in the canonical baseline document, kept bundle-level caveats honest, validated the repo, and advanced Story 7.4 to `review` while leaving Epic 7 open pending retrospective completion.
- 2026-04-08: Story 7.4 accepted as-is and marked `done`; Epic 7 remains open pending retrospective completion.
