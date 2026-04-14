# Implementation Artifacts

Root-level implementation artifacts in this folder represent the active BMAD delivery cycle.

## Current State

- The active root-level implementation tracking artifact is `sprint-status.yaml`.
- Root-level story artifacts track the active playback delivery cycle, including Epic 8 closeout and Epic 9 host-hardening work.
- Root-level tech specs and sprint-change proposals record the active delivery direction; if an older artifact is still useful background but no longer describes the next step, mark it as superseded in place or archive it rather than letting it imply current intent.
- Completed implementation cycles and completed standalone tech specs live under `archive/`.

## Working Rule

Keep only the active sprint's story, tech-spec, status, and retro artifacts at the root.
When an implementation cycle is completed, move its artifact set into `archive/` so the next cycle starts with a clean workspace.

## Archived Implementation Context

- `archive/2026-03-pre-reset-vst-playback-cycle/`: superseded pre-reset VST playback stories, tech specs, and retros that were replaced by the 2026-03-24 plan reset
- `archive/2026-03-multi-flow-score/`: completed multi-flow score implementation cycle
- `archive/2026-03-foundational-tuplet-support/`: completed foundational tuplet implementation cycle, including stories, retros, sprint status, and the follow-on tuplet spacing tech spec
- `archive/2026-03-sample-attack-alignment/`: completed sample attack-alignment implementation tech spec
