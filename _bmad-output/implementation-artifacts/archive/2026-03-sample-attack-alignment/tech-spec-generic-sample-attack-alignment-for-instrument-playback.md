---
title: "Generic Sample Attack Alignment For Instrument Playback"
slug: "generic-sample-attack-alignment-for-instrument-playback"
created: "2026-03-19"
status: "implemented"
stepsCompleted: [1, 2, 3, 4]
tech_stack:
  - "Rust 2021"
  - "Tauri 2"
  - "SCVI sample container pipeline"
  - "symphonia FLAC/Opus decode path"
  - "Repo-local sample attack analysis in Rust build tooling"
files_to_modify:
  - "src-tauri/src/bin/scvi_compress.rs"
  - "src-tauri/src/bin/scvi_inspect.rs"
  - "src-tauri/src/playback/scvi.rs"
  - "src-tauri/src/playback/sampler.rs"
  - "src-tauri/src/playback/streamer.rs"
  - "src-tauri/src/playback/transport.rs"
  - "docs/architecture.md"
  - "docs/development-guide.md"
  - "docs/project-context.md"
code_patterns:
  - "Playback remains Rust-owned and transport state stays silent during playback"
  - "Instrument/sample metadata flows through manifest -> SCVI -> sampler cache -> transport"
  - "Prefer extending existing playback seams over introducing parallel state or transport models"
  - "Current runtime chooses a CacheSample and streams from byte 0 with no sample-start trim or offset metadata"
  - "SCVI sample index is the natural persisted metadata seam for per-sample playback properties"
  - "Streamer is the narrowest runtime seam for trimming analyzed leading frames without redesigning frontend transport"
test_patterns:
  - "Playback regressions should stay close to transport/sampler seams"
  - "Use focused Rust tests for transport and playback helpers"
  - "Extend colocated unit tests in playback modules rather than inventing a new cross-cutting harness"
---

# Tech-Spec: Generic Sample Attack Alignment For Instrument Playback

**Created:** 2026-03-19

## Overview

### Problem Statement

Playback can sound rhythmically wrong even when transport scheduling is correct because different source samples, especially across velocity layers, are not attack-aligned. The current engine streams each selected sample from byte 0 with no sample-start trim or attack-offset compensation, so quieter or differently prepared layers can become audible materially later than louder layers. Raw piano sample inspection already shows onset spread large enough to be heard as timing error, and the fix needs to work generically across staple sampled instruments rather than as a piano-only transport hack.

### Solution

Design and ship a generic sample attack-alignment solution that includes first-class measurement tooling plus runtime playback compensation. The system should quantify effective attack offsets from raw assets and built containers, carry the relevant offset metadata through the existing instrument/container/sampler pipeline, and apply compensation at playback time while proving the behavior with piano and keeping the design reusable for all staple instruments.

### Scope

**In Scope:**

- Generic sample attack-offset analysis and compensation for sampled instruments
- Repo-local tooling to measure/report effective sample attack offsets from source assets and/or built SCVI containers
- Runtime playback support for applying attack alignment without frontend transport redesign
- Validation and proof using the piano library, with the design general enough for all staple instruments
- Developer-facing docs updates if the playback/sample metadata contract changes

**Out of Scope:**

- Piano-only hardcoded transport exceptions
- Analysis-only work that does not produce a shippable playback fix
- Broad frontend transport/state redesign
- Human release workflows, release scripts, or unrelated sample-library cleanup work outside the designed solution path

## Context for Development

### Codebase Patterns

- Playback is Rust-owned under `src-tauri/src/playback/`; transport, sampler, SCVI parsing, and streaming are separate but already connected seams.
- Transport state must remain silent during playback; do not add event-based playback updates or frontend-owned transport truth.
- Existing instrument metadata flows from JSON manifests into SCVI containers and then into the sampler cache; extending this metadata path is more consistent than inventing a parallel offset registry.
- The current sampler path chooses a zone/velocity layer sample and streams from byte 0, so any attack compensation likely belongs in the sample metadata and playback/render path rather than in frontend sequencing.
- Playback regressions should stay near the owning seam: sampler/SCVI tests for metadata carriage, transport/playback tests for compensation behavior.
- The current source-to-runtime flow is: source manifest sample path -> `scvi_compress.rs` META/SIDX/SDAT build -> `ScviContainer::open()` -> sampler cache `CacheSample` -> transport `render_note()` -> streamer `StreamRequest` -> mixer voice.
- `streamer.rs` is the narrowest runtime seam for deterministic sample-start compensation because it already owns decode, resample, pitch shift, and envelope application before audio reaches the mixer.
- There is no existing attack/delay/preroll field anywhere in the manifest, SCVI index, sampler cache, or transport request types, so the spec must define that metadata explicitly rather than rely on a hidden reuse seam.
- `scvi_compress.rs` already walks each source sample while building the container, so attack-offset analysis can be derived there without introducing a second asset-build pipeline or a runtime dependency on external CLIs.

### Files to Reference

| File                                     | Purpose                                                                                               |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `src-tauri/src/playback/streamer.rs`     | Streaming decode path and `StreamRequest`; likely runtime application point for attack skip/alignment |
| `src-tauri/src/playback/sampler.rs`      | Builds sampler cache entries and exposes sample metadata used by playback                             |
| `src-tauri/src/playback/transport.rs`    | Current note scheduling and sample playback dispatch                                                  |
| `src-tauri/src/playback/scvi.rs`         | SCVI container manifest/index reader                                                                  |
| `src-tauri/src/bin/scvi_inspect.rs`      | Existing SCVI inspection tool that can be extended for attack-offset reporting                        |
| `src-tauri/src/bin/scvi_compress.rs`     | SCVI build path where offset metadata may need to be embedded                                         |
| `src-tauri/src/bin/generate_manifest.rs` | Existing manifest-generation tool that may inform or share attack-analysis behavior                   |
| `src-tauri/src/playback/mixer.rs`        | Voice lifecycle and cancellation semantics that constrain runtime compensation design                 |
| `instruments/piano/manifest.json`        | Proof-case instrument manifest with velocity-layer sample mapping                                     |
| `docs/architecture.md`                   | Playback and sampler architecture contract                                                            |
| `docs/project-context.md`                | Guardrails on transport silence, backend ownership, and playback constraints                          |
| `docs/development-guide.md`              | Durable maintainer guidance on keeping playback fixes near owning seams                               |

### Technical Decisions

- Use a hybrid solution: deterministic build-time analysis/reporting plus runtime compensation.
- Make the solution generic across staple instruments, with piano as the explicit proof case and baseline validation target.
- Prefer metadata-driven compensation over transport-local per-instrument hacks.
- Preserve the existing active-flow transport model and frontend command surface; no frontend-owned timing compensation is part of this change.
- Treat this as a shipped playback fix, not just a research task.
- Carry attack alignment as per-sample metadata, not piano-only or transport-only heuristics; round-robin samples and layered assets make per-sample the safer long-term seam.
- Persist `attack_offset_frames` in the existing fixed-size SCVI sample index entry by repurposing reserved bytes rather than inventing a parallel manifest or a second lookup file.
- Store offsets in source-sample-rate frames so the streamer can skip the exact number of decoded source frames before any resample or pitch processing.
- Derive offsets in `scvi_compress.rs` using the existing Rust decode path, with a stable threshold/min-duration detector and a safe fallback of `0` when no qualifying onset is found.
- Thread the metadata through `ScviContainer`, `CacheSample`, and `StreamRequest`, then apply the trim in `streamer.rs` before audio reaches the mixer.
- Include first-class inspection/reporting in the repo so maintainers can measure built-container offsets and verify the piano proof case without ad hoc shell scripts.
- Existing transport warmup behavior may remain in place, but attack alignment must not depend on adding new frontend scheduling or transport state.

## Implementation Plan

### Tasks

- [x] Task 1: Add build-time attack-offset analysis to the SCVI builder
  - File: `src-tauri/src/bin/scvi_compress.rs`
  - Action: Add a deterministic helper that decodes each source sample during container build, detects the first sustained audible frame using a documented threshold/min-duration rule, and records that value as `attack_offset_frames` for the sample being written into the SCVI index.
  - Notes: Reuse the existing Rust decode path instead of introducing an `ffmpeg` runtime/build dependency; treat missing or ambiguous detection as offset `0`, and keep logging/reporting explicit enough to diagnose bad detections during asset rebuilds.

- [x] Task 2: Persist and load per-sample attack offsets through SCVI metadata
  - File: `src-tauri/src/bin/scvi_compress.rs`
  - Action: Write `attack_offset_frames` into the reserved bytes of the fixed-size sample index entry so the container record size stays stable.
  - File: `src-tauri/src/playback/scvi.rs`
  - Action: Extend the runtime `SampleIndex` model and parser to read the stored attack offset and expose it to downstream playback code.
  - Notes: Keep older containers readable by defaulting missing/reserved-zero values to `0`; document that rebuilt `.scvi` assets are required to benefit from the new metadata, but no `.soloc` migration is needed because project files do not persist SCVI sample-index contents.

- [x] Task 3: Thread attack offsets through sampler cache selection and playback requests
  - File: `src-tauri/src/playback/sampler.rs`
  - Action: Add `attack_offset_frames` to `CacheSample` and carry it from `SampleIndex` into the cached sample metadata used at note-render time.
  - File: `src-tauri/src/playback/transport.rs`
  - Action: Include the cached attack offset when constructing the playback request for a scheduled note.
  - File: `src-tauri/src/playback/streamer.rs`
  - Action: Extend `StreamRequest` to accept `attack_offset_frames` and prepare the streaming path to honor it.
  - Notes: Keep the active-flow transport model unchanged; this task only threads metadata through the existing Rust-owned playback path.

- [x] Task 4: Apply sample-start trimming in the streamer without changing transport ownership
  - File: `src-tauri/src/playback/streamer.rs`
  - Action: Discard exactly `attack_offset_frames` decoded source frames before feeding audio into resample/pitch/envelope processing, with correct channel handling and zero-offset fast path behavior.
  - Notes: The trim must occur before resampling so source-rate frame counts stay exact; ensure cancellation, short-sample handling, and end-of-stream logic still behave correctly when the requested trim consumes all available decoded frames.

- [x] Task 5: Add built-container inspection and piano proof reporting
  - File: `src-tauri/src/bin/scvi_inspect.rs`
  - Action: Extend the inspector output to print each sample's stored `attack_offset_frames` and a human-readable duration so maintainers can verify rebuilt containers.
  - File: `src-tauri/src/bin/scvi_compress.rs`
  - Action: Emit a concise build summary for analyzed offsets so the piano proof case can be validated during rebuilds without separate ad hoc scripting.
  - Notes: The inspection/report path should make it straightforward to compare piano velocity layers for the same pitch and confirm that previously late layers now carry meaningful non-zero offsets.

- [x] Task 6: Add focused regression coverage for metadata carriage and playback compensation
  - File: `src-tauri/src/playback/scvi.rs`
  - Action: Add tests that parse sample index entries with zero and non-zero attack offsets and assert correct defaults/decoded values.
  - File: `src-tauri/src/playback/streamer.rs`
  - Action: Add tests proving the streamer skips the requested source frames, preserves zero-offset behavior, and handles offsets that equal or exceed the decoded sample length.
  - File: `src-tauri/src/playback/transport.rs`
  - Action: Extend targeted transport/request-construction tests to verify the selected sample's attack offset is passed into the playback request without altering existing scheduling semantics.
  - Notes: Keep tests close to the owning seams; do not introduce a new cross-cutting playback harness unless an existing unit seam proves insufficient.

- [x] Task 7: Update architecture and maintainer documentation
  - File: `docs/architecture.md`
  - Action: Document the new sample attack-alignment metadata flow from SCVI build through runtime playback.
  - File: `docs/development-guide.md`
  - Action: Document how maintainers rebuild assets, inspect stored offsets, and validate playback alignment for proof instruments such as piano.
  - File: `docs/project-context.md`
  - Action: Record the architectural rule that sample attack alignment is a backend-owned, metadata-driven playback concern and must not be reimplemented as frontend timing state.
  - Notes: This task is mandatory because the change alters playback architecture, developer workflow, and the sample metadata contract.

### Acceptance Criteria

- [ ] AC 1: Given a source sample with measurable leading silence, when the SCVI builder processes it, then the generated sample index stores a non-zero `attack_offset_frames` value derived from the documented detector instead of silently dropping the measurement.
- [ ] AC 2: Given an older or zero-offset SCVI sample entry, when the runtime container loader parses it, then playback metadata defaults safely to `attack_offset_frames = 0` and existing samples remain playable without a project-file migration.
- [ ] AC 3: Given a selected cached sample with a non-zero attack offset, when transport creates the playback request, then the request carries that exact offset through to the streamer without altering note scheduling timestamps or frontend transport state.
- [ ] AC 4: Given a `StreamRequest` with `attack_offset_frames`, when the streamer begins decoding, then it discards exactly that many source-rate frames before resampling/output and produces identical behavior to current playback when the offset is `0`.
- [ ] AC 5: Given an offset that is equal to or larger than the decoded source length, when the streamer processes the request, then it exits cleanly without panic, buffer corruption, or stuck voices.
- [ ] AC 6: Given a rebuilt piano SCVI and the inspection/reporting commands, when a maintainer compares representative layers for the same pitch, then the reported offsets clearly expose the previously late quiet layers and provide a repeatable proof that the runtime now has the metadata needed to align them.
- [ ] AC 7: Given the new playback metadata path, when the focused Rust tests run, then they cover SCVI parsing, offset propagation, zero-offset fallback, and streamer trimming edge cases without introducing a separate playback-test framework.
- [ ] AC 8: Given the architecture and maintainer docs, when a fresh developer reads them, then they can explain where attack offsets are derived, where they are stored, how they are applied at runtime, and how to validate the piano proof case.

## Additional Context

### Dependencies

- Existing Rust playback stack: Tauri, symphonia decode, SCVI container tooling, sampler cache path, and transport/streamer unit-test seams
- SCVI container rebuild workflow for bundled sample libraries; updated attack metadata only appears after rebuilding `.scvi` assets
- No new frontend API dependency and no `.soloc` migration dependency; the change is isolated to instrument asset metadata and Rust playback consumption
- Optional local `ffmpeg`/`ffprobe` usage remains a developer validation aid only, not a required implementation dependency

### Testing Strategy

- Extend Rust playback and sampler-adjacent tests near the owning seams.
- Add proof-oriented validation for piano attack alignment and generic metadata carriage behavior.
- Preserve existing non-timing transport behavior while proving compensated playback closes the measured onset gap caused by leading silent frames.
- Use colocated unit tests in `scvi.rs`, `transport.rs`, and `streamer.rs` as the primary automated coverage.
- Rebuild the piano `.scvi`, inspect stored offsets for representative pitches/layers, and manually audition repeated notes across `ppp`, `mp`, `f`, and `ff` to confirm audible alignment improves without transport regressions.
- If the builder detector is sensitive to instrument-specific noise floors, add deterministic fixture-style unit coverage around the detector helper before widening rollout to additional staple libraries.

### Notes

- Quick validation already showed piano onset spread across velocity layers large enough to be perceptible, so implementation should optimize for shipping the metadata/runtime fix rather than re-litigating whether the issue exists.
- The highest-risk technical decision is detector robustness for instruments with intentional breath/noise attacks; the safe fallback is `attack_offset_frames = 0`, with any future manual override mechanism treated as follow-up scope if real libraries prove the detector insufficient.
- Because the SCVI sample index changes semantically, rebuilt instrument assets are required for the new behavior; however, this does not change persisted score/project semantics and therefore does not require a `.soloc` migration.
- Existing transport warmup/grouping changes can remain as independent scheduling hardening, but they are not the primary fix for sample-layer attack misalignment.
- Future work, if needed, can add a source-manifest override for rare false positives, but the initial shipped solution should stay minimal: deterministic build-time analysis, persisted per-sample metadata, runtime trim, tests, and documentation.
