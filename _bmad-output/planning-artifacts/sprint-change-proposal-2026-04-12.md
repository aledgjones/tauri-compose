# Sprint Change Proposal

Date: 2026-04-12
Project: compose
Change scope: Moderate
Workflow: Correct Course
Mode: Batch
Recommended path: Direct Adjustment + Focused Epic 9 Rebaseline
Approval status: Approved and applied
Resolution status: Epic 9 artifacts updated to treat real host-owned contract hardening as Story 9.3 and continued real-host remediation toward the HALION product path as Story 9.4.

## 1. Issue Summary

### Triggering story and discovery

- Triggering story: Story 9.2 - Extract A Stable Backend-Owned VST3 Host Core From The POC.
- Triggering discovery: automated HALION load/init reruns and crash-report inspection on 2026-04-12 after the host-core extraction landed.

### Core problem statement

Story 9.2 did the right structural work. It extracted a real backend-owned host core and several 2026-04-12 reruns removed genuine host bugs around callback-pointer handling, `getName`, and `createInstance`. But the remaining HALION failure boundary no longer looks like a small timing tweak or a simple result-code issue.

The latest private-GUID aliasing and opaque-stub experiments were useful diagnostics, yet they are not converging on a credible fix path. The user direction is now explicit: keep HALION as the acceptance bar, stop treating fake-success shims as progress, and focus Epic 9 on real host-owned structures and flows that HALION-class plugins reasonably expect.

### Evidence

- Story 9.2 extracted `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` and made ordinary editor-host opens plus maintainer probes share one lifecycle owner.
- Subsequent live reruns moved the crash boundary through multiple real host bugs, which confirms the investigation produced useful evidence.
- The latest bounded experiment still ended in a plugin-side crash after private-GUID handling changes, which suggests guessed opaque success paths are not the right long-term lever.
- The top-level Epic 9 summary, Story 9.3, Story 9.4, sprint status, and active research/spec artifacts still partly reflect the older investigation-first and scanner-lane framing.

## 2. Impact Analysis

### Epic impact

- Epic 9 remains the correct home for this work.
- Stories 9.1 and 9.2 remain valid foundation slices and should not be reopened as mistakes.
- Story 9.3 should be repurposed from a scanner-style compatibility lane into real host-owned init-contract hardening.
- Story 9.4 should no longer stop at rerun-and-evidence publication. It should continue adding real host-owned abilities on the shared seam until HALION reaches the bounded ordinary product path, while keeping one known-good control bundle in the regression matrix.

### Story and artifact impact

- `_bmad-output/planning-artifacts/epics.md` had a stale top-level Epic 9 title and outdated Story 9.3 and 9.4 text.
- `_bmad-output/implementation-artifacts/sprint-status.yaml` still tracked the old 9.3 and 9.4 story slugs.
- `_bmad-output/implementation-artifacts/README.md` still implied there were no active root-level story or tech-spec artifacts.
- `_bmad-output/implementation-artifacts/9-1-classify-reference-multi-instrument-plugin-load-and-initialization-failures.md` still described Epic 9 as investigation-only.
- `_bmad-output/implementation-artifacts/9-2-extract-a-stable-backend-owned-vst3-host-core-from-the-poc.md` needed a follow-on note so the recorded private-GUID experiments do not read as the endorsed path forward.
- `_bmad-output/implementation-artifacts/tech-spec-investigate-multi-instrument-vst-support-with-halion.md` still recommended a scanner-style next step.

### Technical impact

- `src-tauri/src/playback/plugin_editor_poc/macos_host/vst3_host_core.rs` remains the canonical seam for follow-on hardening.
- Probe-era private-GUID aliases, opaque stubs, and related fake-success branches become cleanup targets or bounded diagnostic residue rather than the main implementation strategy.
- The ordinary product path plus shared host-core logging becomes the only rerun harness; no retained probe-only lifecycle or parallel probe path should remain.

### Documentation and instruction impact

- A focused spec is required for each active Epic 9 implementation slice so the next remediation step stays explicit instead of collapsing back into vague probe work.
- AI-agent instructions already capture the durable Epic 9 rule: prefer real public-contract hardening over fake-success private-GUID shims. This course correction changes story scope, not that durable rule.
- Implementation docs such as `docs/architecture.md`, `docs/development-guide.md`, and `docs/playback-support-baseline.md` should update when the hardened rerun lands.

## 3. Recommended Approach

Selected approach: Direct adjustment with a narrow Epic 9 rebaseline.

1. Keep Stories 9.1 and 9.2 as valid completed or in-review foundation work.
2. Reframe Story 9.3 as real host-owned init-contract hardening with cleanup-in-place.
3. Reframe Story 9.4 as continued real host-owned ability work against the hardened seam until HALION works in the bounded product path, with reruns and evidence publication serving that implementation goal.
4. Add a focused implementation spec now so the next slice stays tight.

### Rationale

- This is not a product pivot and not a new epic.
- The investigation already did its job: it classified the failure and exposed the need for a better host seam.
- The cleanest move is to keep the extracted host core as the canonical seam, stop extending fake-success shims, and make Epic 9 progress depend on real host-owned contract work.

### Effort, risk, and timeline

- Effort: Medium.
- Risk: Medium to high, because HALION may still expose a narrower remaining blocker after public-contract hardening.
- Timeline impact: Narrow. This changes the next slice, not the whole plan.

## 4. Detailed Change Proposals

### Proposal 1: Rebaseline Epic 9 around host hardening

- Update the top-level Epic 9 title and scope note in `epics.md` so they match the detailed epic direction already taking shape in Story 9.2.
- Keep HALION as the acceptance bar.
- Make explicit that route-choice, shared-slot, and multi-instrument claims stay blocked until bounded non-crashing init and later capability proof exist.

### Proposal 2: Replace the stale Story 9.3 plan

- Remove the scanner-style compatibility-lane framing as the next story.
- Replace it with real host-owned init-contract hardening that grows the shared host core and cleans up superseded probe-era shim branches.

### Proposal 3: Reframe Story 9.4 as the HALION product-path remediation step

- Make Story 9.4 continue adding real host-owned abilities on the shared host core until HALION reaches the ordinary bounded product path.
- Keep one known-good single-instrument control bundle in the same rerun matrix so the proven path stays protected.
- Treat ordinary-path runs plus shared host-core logging as proof gates inside the story, not a separate probe workflow.

### Proposal 4: Add a focused implementation spec

- Create `tech-spec-halion-host-contract-hardening.md` as the immediate guidance artifact for Story 9.3.
- Make documentation impact explicit inside the spec.
- Keep the scope tight: real host-owned public-contract hardening first, capability probing later.
- Follow Story 9.3 with a dedicated Story 9.4 implementation spec or story artifact that turns the new product-path remediation goal into concrete host-seam tasks and proof gates.

### Proposal 5: Update durable repo guidance

- Add a rule to `.github/copilot-instructions.md` that current Epic 9 host work must prefer real host-owned public-contract hardening and must not count guessed private-GUID success paths as the main route to HALION compatibility.

## 5. Implementation Handoff

### Change scope classification

Moderate.

### Handoff recipients

- Development team or dev agent for Story 9.4 implementation.
- Architecture-aware reviewer for the host-contract seam and ordinary-product-path boundary.

### Responsibilities

- Implement the focused Story 9.3 host-contract hardening against the shared host core.
- Remove or quarantine superseded probe-era shim branches as real host-owned seams replace them.
- Create or update the Story 9.4 implementation spec so product-path remediation tasks, constraints, and proof gates are explicit before coding begins.
- Implement Story 9.4 host-seam additions against the shared backend-owned host core until HALION reaches the bounded ordinary product path or the remaining blocker becomes specific enough for a narrower follow-on slice.
- Use ordinary-path runs plus shared host-core logging as proof gates for each meaningful host-seam change.
- Update docs, support posture, and blocked-bundle policy only when the product-path evidence changes.

### Success criteria

- Epic 9 artifacts, sprint status, and the focused spec all describe the same next slice.
- Story 9.3 remains the cleanup-and-hardening slice, and Story 9.4 now owns the next real host-owned ability work against HALION.
- HALION remains the acceptance bar while current user-facing support posture stays blocked until ordinary product-path evidence justifies changing it.
