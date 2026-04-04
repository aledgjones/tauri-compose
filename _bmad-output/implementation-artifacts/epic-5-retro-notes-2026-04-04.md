# Epic 5 Retro Notes

Date: 2026-04-04
Epic: 5 - Trust Playback During Real Composition
Mode: in-flight notes for later retrospective pickup

## Why This Note Exists

- Epic 5 planning had a major story-sequencing defect: Story 5.1 had already been developed against an assumed audible transport baseline that the story sequence had never explicitly established.
- The PRD and higher-level planning docs already treated real transport-driven playback as foundational, but the original Epic 5 story list left that prerequisite implicit at the story level.
- This note is meant to preserve that process lesson for the Epic 5 retrospective instead of relying on memory later.

## Retro Candidate

- Major planning failure: the Epic 5 story progression implied foundational score playback instead of explicitly defining it before trust-hardening, seek, and editing stories.

## Evidence

- Story 5.1 was created and implemented as transport trust-hardening on top of an assumed already-audible transport path.
- Later review of the PRD and architecture showed that Epic 5 itself, not Epic 4 or Epic 6, needed an explicit story to make transport drive real routed score playback.
- The plan was corrected by inserting Story 5.0, moving the audible transport baseline into Epic 5, and tightening Stories 5.1 through 5.5 so they explicitly extend that baseline.
- As a result, Story 5.1 was moved from `review` to `blocked` pending Story 5.0, even though its implementation work remains potentially reusable once the missing prerequisite lands.
- Story 5.2 was later rewritten against the corrected baseline rather than merely continuing unchanged from the original sequence.
- Story 5.2a was added as a fill-in hardening story to cover playback clocking and runtime reset semantics that the original progression had not carved out clearly enough.

## Severity

- This was not a minor wording gap or routine mid-epic refinement.
- This was a dependency-modeling failure in the story inventory for an architecture-sensitive epic.
- The defect was dangerous because downstream stories still looked locally reasonable if the missing playback foundation was silently assumed.
- Higher-level product and architecture intent were directionally correct; the failure was in decomposing that intent into an explicit, safe execution order.

## Lesson To Carry Into Retro

- When a capability is foundational in the PRD or architecture, the story sequence must name that prerequisite explicitly instead of assuming it through later-story wording.
- Playback stories need explicit story-level statements for foundation seams such as "transport is already audibly driving the routed product path" before trust-hardening or realism stories start.
- Review-ready implementation can still be directionally correct and yet be sequenced incorrectly if the story inventory leaves a prerequisite implicit.

## Retro Framing

- Primary Epic 5 process failure: incorrect story dependency modeling.
- Not a failure: Epic 5 remaining open until retrospective completion. That matches the intended closeout workflow.
- Secondary process or template issues should stay out of the main retro narrative unless they materially changed delivery.

## Candidate Action Items

1. Add a story-preparation check that compares each story's acceptance criteria against any prerequisite capabilities stated in the PRD and architecture.
2. For architecture-sensitive epics, require the first story that establishes a foundational runtime path to be explicit in the story list instead of implied by downstream trust or UX stories.
3. During story creation, flag any acceptance criteria that assume an already-existing playback capability without naming the story or seam that establishes it.
4. Classify architecture-sensitive stories as foundation, extension, or hardening stories during preparation so dependency order is visible before implementation starts.
