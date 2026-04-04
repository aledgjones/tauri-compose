# Epic 5 Retro Notes

Date: 2026-04-04
Epic: 5 - Trust Playback During Real Composition
Mode: in-flight notes for later retrospective pickup

## Why This Note Exists

- Epic 5 planning was corrected mid-stream after Story 5.1 had already been developed against an assumed audible transport baseline.
- The PRD and higher-level planning docs already treated real transport-driven playback as foundational, but the original Epic 5 story list did not make that prerequisite explicit enough at the story level.
- This note is meant to preserve that process lesson for the Epic 5 retrospective instead of relying on memory later.

## Retro Candidate

- Story-level playback prerequisites were not explicit enough even though the planning docs already carried the requirement.

## Evidence

- Story 5.1 was created and implemented as transport trust-hardening on top of an assumed already-audible transport path.
- Later review of the PRD and architecture showed that Epic 5 itself, not Epic 4 or Epic 6, needed an explicit story to make transport drive real routed score playback.
- The plan was corrected by inserting Story 5.0, moving the audible transport baseline into Epic 5, and tightening Stories 5.1 through 5.5 so they explicitly extend that baseline.
- As a result, Story 5.1 was moved from `review` to `blocked` pending Story 5.0, even though its implementation work remains potentially reusable once the missing prerequisite lands.

## Lesson To Carry Into Retro

- When a capability is foundational in the PRD or architecture, the story sequence must name that prerequisite explicitly instead of assuming it through later-story wording.
- Playback stories need explicit story-level statements for foundation seams such as "transport is already audibly driving the routed product path" before trust-hardening or realism stories start.
- Review-ready implementation can still be directionally correct and yet be sequenced incorrectly if the story inventory leaves a prerequisite implicit.

## Candidate Action Items

1. Add a story-preparation check that compares each story's acceptance criteria against any prerequisite capabilities stated in the PRD and architecture.
2. For architecture-sensitive epics, require the first story that establishes a foundational runtime path to be explicit in the story list instead of implied by downstream trust or UX stories.
3. During story creation, flag any acceptance criteria that assume an already-existing playback capability without naming the story or seam that establishes it.
