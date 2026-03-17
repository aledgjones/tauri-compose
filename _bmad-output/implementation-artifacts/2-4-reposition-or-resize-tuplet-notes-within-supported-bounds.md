# Story 2.4: Reposition or Resize Tuplet Notes Within Supported Bounds

Status: in-progress

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a composer,
I want to drag tuplet notes to move, re-pitch, or resize them within the tuplet frame,
so that I can refine melodic and rhythmic shape without breaking exact tuplet semantics.

## Scope Note

- This story is about gesture-driven editing of existing tuplet-owned tones in the sequencer.
- Supported gesture families are: horizontal reposition, vertical re-pitch during drag, right-edge resize, and left-edge resize that changes start tick plus duration.
- Supported edits must preserve exact timing, one-stored-event-per-tone ownership, and single-level MVP tuplet structure.
- When a tone's start crosses the tuplet boundary, ownership must move with the start position in both directions: tuplet-owned tones become track-owned when they start outside the tuplet, and track-owned tones become tuplet-owned when they start inside the tuplet.
- Tuplet containers themselves are not resizable in this story.
- Snap duration remains literal across the boundary. If the active snap is an eighth note, outside the tuplet the gesture snaps to track eighths and inside the tuplet it snaps to tuplet eighths; the written duration choice stays the same while projection through time changes by context.
- Re-pitch is in scope when it happens through the existing drag gesture. Standalone pitch editing parity already exists from Story 2.2.
- Slot add/remove behavior remains Story 2.3 scope. Container delete and slice semantics remain Story 2.5 scope.
- All editing remains single-tone only. Multi-tone editing is not in the app yet and must not be introduced by this story.
- This story must not introduce frontend-owned tuplet edit state, nested tuplets, dissolve-to-host-time behavior, or a tuplet-only drag model separate from ordinary tone gestures.

## Correct Course Update 2026-03-17

- The previous Story 2.4 implementation path over-invested in boundary-specific cursor translation and pixel-to-tick repair logic. That path is no longer considered the correct architectural direction for this story.
- The corrected direction is to introduce one authoritative sequencer address lattice for rendering, hit-testing, and snapping. This lattice must model the sequencer as an ordered continuous address space whose entries are semantic edit boundaries rather than host-tick pixels with tuplet exceptions layered on top.
- Tuplet-covered spans replace the host-track span inside their covered interval. The canonical frontend projection model must not translate tuplet-local ticks into host ticks or approximate boundary math with float repair logic.
- Each canonical address is a pair of local context and local tick, for example `{ tuplet_id: null, tick: 96 }` or `{ tuplet_id: 11, tick: 16 }`. The address stays local to its owning context.
- The canonical lattice should be represented as an ordered continuous array of boundary points plus fast lookup helpers keyed by address. String-encoded address keys are preferred over object-identity map keys.
- The active snap duration must not define canonical geometry. A snap-specific sparse index or derived overlay is acceptable for performance, but rendering and hit-testing must still derive from the canonical address lattice.
- The top tick bar remains a host-track ruler only. It continues to display ordinary track ticks, beats, and bars without tuplet-local scaling.
- The tuplet container rendering remains the visual cue for local subdivision differences. Its tinted region, annotation, and inner beat presentation already communicate where spacing is morphed.
- The sequencer interaction layer must stop treating the host tick ruler as the implicit snapping authority with tuplet exceptions layered on top. Rendering, hit-testing, and snap resolution should all derive from the same canonical address model.
- This is a controlled reset of the implementation approach, not a change to the product intent. The acceptance criteria remain valid, but the implementation notes and tasks below supersede the previous wrong-path assumptions.

## Acceptance Criteria

1. Given a tuplet note is edited for position or duration inside the tuplet, when the requested edit remains within the tuplet's supported local frame rules, then the change is applied using exact local timing and the result remains a valid tuplet state.
2. Given a tuplet note is dragged vertically using an existing pitch-drag gesture, when the requested edit stays within supported MVP bounds, then the product updates pitch using the same pitch-targeting rules used for ordinary track tones and tuplet membership remains an implementation detail rather than a separate drag behavior model.
3. Given a drag gesture changes pitch together with position or duration in a supported MVP case, when the edit is committed, then the product preserves the same user-facing drag semantics expected outside tuplets while applying the correct tuplet-local timing and ownership rules underneath and the gesture is not limited to timewise movement only.
4. Given a tone duration is dragged across a tuplet boundary in a supported MVP case, when the edit is committed, then the product preserves one continuous persisted tone and derives the boundary split at render or playback time and the user-facing gesture remains one continuous duration edit rather than a manual delete-and-recreate workflow.
5. Given a tone's start position is dragged across the tuplet boundary, when the edit is committed, then ownership moves to the context where the tone now starts, including tuplet-to-track and track-to-tuplet transfer as applicable, and the tone still remains exactly one stored event.
6. Given an edit would create unsupported overlap or otherwise violate MVP rules, when the composer attempts that edit, then the product rejects the unsupported outcome explicitly and it does not silently convert the passage into approximate non-tuplet timing.
7. Given the user attempts an unsupported tuplet edit in the sequencer, when the edit is rejected, then the product follows the existing product rejection behavior without introducing tuplet-specific error messaging and the tuplet region remains visually stable enough for the user to recover without losing orientation.
8. Given a tuplet sits next to non-tuplet material, when a supported move or resize occurs near the boundary, then ownership across the boundary remains explicit and hit targeting and selection remain stable.
9. Given a tone already exists before a tuplet is created and the composer later extends it across the tuplet area, when the resize completes, then the resulting host and tuplet segments preserve one coherent musical intent across the boundary and the tuplet's visible inner beats and boundary remain legible during and after the edit.
10. Given an edit would require nested tuplet structure to remain musically valid, when the composer attempts that edit, then MVP rejects or defers the edit rather than creating nested tuplets implicitly.
11. Given tuplet drag and resize behavior changes documented editing semantics or developer guardrails, when the story is completed, then the relevant docs and agent instructions are updated in the same change, or the completion notes explicitly record why no documentation update was required.
12. Given the active snap duration is a literal written value such as an eighth note, when the composer drags or resizes a tone across a tuplet boundary, then snapping continues to use that same literal written value in each context, using tuplet-local eighths inside the tuplet and track eighths outside it.
13. Given the sequencer contains tuplets, when the composer edits notes in this story, then only tone editing is supported and the tuplet container itself is not resized.
14. Given the app has no multi-tone editing support yet, when Story 2.4 is implemented, then all gesture handling remains single-tone only and no grouped move or resize behavior is introduced.

## Tasks / Subtasks

- [ ] Replace the wrong-path frontend projection model with an authoritative sequencer address lattice (AC: 1, 4, 5, 8, 9, 12, 13)
  - [ ] Build a shared sequencer address structure whose canonical ordered array represents semantic edit boundaries across the full track, including tuplet-local regions.
  - [ ] Ensure a tuplet span replaces the host span inside its covered interval in the canonical array. Do not maintain parallel host and tuplet address tracks for the same horizontal region.
  - [ ] Introduce stable address keys such as `host:96` or `tuplet:11:16` for direct address lookup. Do not rely on object identity for address maps.
  - [ ] Keep the top tick bar visual-only. It remains a host-track ruler and must not become the snapping authority for gesture math.
  - [ ] Keep the tuplet container rendering as the visual cue for morphed local spacing; do not redesign it into a second ruler.
- [ ] Rework shared sequencer drag handling to consume the address lattice for projection and hit-testing (AC: 1, 2, 3, 4, 5, 8, 12)
  - [ ] Update `src/windows/main/components/sequencer-track.tsx` so tone rendering, move, right-edge resize, and left-edge resize all resolve through address lookups and address-span projection rather than width multiplication.
  - [ ] Preserve shared gesture semantics for host-owned and tuplet-owned tones rather than creating a tuplet-only drag stack.
  - [ ] Keep all gesture handling single-tone only; do not introduce grouped selection or multi-tone drag logic.
  - [ ] Keep pitch drag on the existing shared pitch-edit path; only the horizontal projection, hit-testing, and snap resolution model should change.
- [ ] Keep the canonical geometry independent of active snap while allowing derived snap indexes (AC: 1, 4, 8, 12)
  - [ ] Ensure rendering and hit-testing use the full canonical address lattice rather than a snap-filtered subset.
  - [ ] Allow a derived sparse snap index or overlay keyed off the canonical lattice if needed for performance, but do not let snap duration define the stored geometry.
  - [ ] Ensure existing note edges remain addressable even when the current snap duration is coarser than the stored note boundaries.
- [ ] Preserve backend-owned validity and ownership transfer rules at commit time (AC: 1, 4, 5, 6, 10, 12)
  - [ ] Re-evaluate the current Rust and action seam changes against the corrected address-lattice design and keep only the parts still required for ownership transfer and backend validation.
  - [ ] Preserve the rule that crossing the tuplet boundary does not force a storage split; one persisted tone remains authoritative and boundary splitting stays derived.
  - [ ] Implement ownership transfer in both directions when the committed start position crosses the tuplet boundary: child-to-host when the start leaves the tuplet, host-to-child when the start enters it.
  - [ ] Keep snapping literal across the boundary so a single snap choice maps to host durations outside tuplets and equivalent tuplet-local durations inside them.
- [ ] Keep sequencer interaction and visual recovery stable for accepted and rejected edits (AC: 6, 7, 8, 9)
  - [ ] Ensure rejected edits leave the tuplet region, slot markers, boundary lines, and tone selection stable enough that the user can continue working without losing orientation.
  - [ ] Keep empty tuplet space as interaction context, not a selectable synthetic rest or separate edit target.
  - [ ] Keep tuplets themselves non-resizable; all resize work in this story applies only to tones.
- [ ] Add focused regression coverage for the corrected address-lattice behavior (AC: 1, 2, 3, 4, 5, 6, 8, 9, 10, 12)
  - [ ] Cover host-owned and tuplet-owned tone rendering against the same canonical address projection model.
  - [ ] Cover move, left resize, and right resize across host-only, tuplet-only, and boundary-crossing cases.
  - [ ] Cover canonical address lookup, span projection, and nearest-snap resolution independently of React rendering.
  - [ ] Cover rejection when an edit would require unsupported nested-tuplet repair or other invalid local-frame outcomes.
  - [ ] Extend backend tests only for the final committed seam behavior that survives the architectural reset.
- [ ] Validate the touched seams with the standard repo checks
  - [ ] Run the relevant frontend tests covering sequencer tuplet drag and resize behavior.
  - [ ] Run the relevant Rust tests covering tuplet-aware tone movement and duration updates.
  - [ ] Run `npm run lint` and `npm run check-types` after implementation.
  - [ ] Run `cargo test --manifest-path src-tauri/Cargo.toml` if Rust tone or track mutation seams change.
  - [ ] Run `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features` if Rust files change, because clippy-clean is a repo rule.
- [ ] Review documentation impact before closing the story (AC: 11)
  - [ ] Confirm whether the shipped drag and resize rules change documented tuplet editing semantics, rejection boundaries, or developer guardrails.
  - [ ] Update the relevant docs and agent instructions in the same change if the contract changed materially.
  - [ ] If no docs change is needed, record that explicit decision in the completion notes.

## Dev Notes

### Story Foundation

- Epic 2 is about editing tuplets without breaking musical intent. Story 2.4 is the gesture-editing story after targeting, pitch parity, and slot add/remove groundwork.
- Story 2.1 established reliable targeting of tuplet regions and child tones.
- Story 2.2 established that tuplet-owned pitch changes should flow through the same shared tone seams as ordinary tones.
- Story 2.3 established that note add/remove inside tuplets should reuse shared sequencer and Rust entry-management seams with explicit `tuplet_id` context.
- Story 2.4 should extend the same philosophy to drag editing: shared gesture semantics, backend-owned truth, and tuplet-local validation rather than a parallel tuplet editing mode.

### Technical Requirements

- Reuse the existing frontend drag handlers in `src/windows/main/components/sequencer-track.tsx`:
  - `handlePositionDragging(...)`
  - `handleDurationAdjustment(...)`
  - `handleLeftSideAdjustment(...)`
  - `handleSelectTool(...)`
- Introduce a shared sequencer address-lattice seam for Story 2.4. This seam should become the authoritative source for:
  - semantic-address to pixel projection
  - pixel to nearest valid semantic address resolution
  - cross-boundary hit-testing for move and resize gestures
  - tone start and end geometry across host and tuplet regions
- The canonical frontend model should be an ordered continuous array of semantic boundary points. Each point represents an address such as `{ tuplet_id: null, tick: 96 }` or `{ tuplet_id: 11, tick: 16 }` paired with its x position in the sequencer.
- Tuplet-covered spans replace host spacing within their covered interval rather than creating a second parallel hit-test track. The canonical model must not translate tuplet-local ticks into host ticks or rely on float repair logic to recover boundaries.
- Direct address lookup should use stable encoded keys rather than object-identity map keys.
- If performance requires it, a sparse snap index may be derived from the canonical lattice for the current snap duration. That sparse snap index is an optimization only; it must not replace canonical geometry.
- The tick bar remains host-track visual reference only. It must not be reused as the true snapping authority when tuplets are present.
- Reuse the existing frontend action wrappers in `src/services/actions.ts`:
  - `actions.score.tone.shift(track_id, entry_id, new_tick)`
  - `actions.score.tone.setDuration(track_id, tone_id, duration)`
  - `actions.score.tone.setPositionAndDuration(track_id, tone_id, tick, duration)`
  - `actions.score.tone.setPitch(track_id, tuplet_id, tone_id, pitch_int, accidental?)`
- Reuse and refine the existing backend tone seams in `src-tauri/src/score/entries/tone.rs`:
  - `Track::set_tone_duration(...)`
  - `Track::set_tone_position_and_duration(...)`
  - `score_entries_tone_duration_set(...)`
  - `score_entries_tone_set_position_and_duration(...)`
  - `score_entries_tone_shift(...)`
  - `resolve_tuplet_pitch_tick(...)`
- Reuse shared track indexing logic in `src-tauri/src/score/tracks.rs`, but add backend validation where tuplet-owned movement would otherwise escape or invalidate the local frame.
- Preserve exact timing and one-stored-event-per-tone ownership. A drag may cross a tuplet boundary, but the committed result must still be one stored tone whose ownership is determined by where it starts.
- Preserve literal duration semantics across the boundary. An eighth-note snap remains an eighth-note choice whether the edited segment is inside or outside the tuplet; only the projection into performed host time changes.
- Rendering a duration must be address-span based rather than width-multiplication based. A tone that crosses a boundary may consume multiple local spacing regimes while remaining one stored event.
- Preserve MVP non-nestable rules. Any drag result that would require nested tuplets or nested repair must be rejected or deferred.

### Expected Data Structures

- The canonical frontend coordinate model for this story should be built around an address type, not around host-tick projection with tuplet exceptions. Expected shape:

```ts
type AxisAddress = {
  tuplet_id: UniqueId | null;
  tick: number;
};
```

- The canonical ordered lattice should store boundary points, not arbitrary sampled pixels. Expected shape:

```ts
type AxisPoint = {
  address: AxisAddress;
  x: number;
};
```

- The canonical lattice should be represented as one ordered continuous array covering the full editable horizontal space. Expected shape:

```ts
type SequencerAddressLattice = {
  continuous: AxisPoint[];
  byAddressKey: Map<string, number>;
};
```

- `continuous[i]` represents the x position of a valid semantic boundary in the sequencer. Advancing from index `i` to `i + 1` means advancing by one semantic tick in the local owning context of that boundary.
- Tuplets replace the covered host span in `continuous`. The canonical array must not contain overlapping host and tuplet entries for the same horizontal region.
- The canonical lattice must be built at the persisted semantic tick resolution so that existing note starts and ends remain addressable regardless of the currently selected snap duration.
- `byAddressKey` must use stable encoded keys rather than object identity. Expected encoding examples:

```ts
const hostKey = "host:96";
const tupletKey = "tuplet:11:16";
```

- Do not use a `Map` keyed directly by `AxisAddress` objects. JavaScript object identity would make that lookup unstable for equivalent addresses.
- Do not rely on a `Map` keyed directly by floating-point x values for canonical lookup. X-based lookup should use the ordered `continuous` array with nearest/floor/ceiling binary-search helpers.

- A derived snap overlay is allowed, but it must be downstream of the canonical lattice. Expected shape:

```ts
type SnapPoint = {
  continuousIndex: number;
  address: AxisAddress;
  x: number;
};

type SequencerSnapOverlay = {
  ticksPerSnap: number;
  points: SnapPoint[];
};
```

- The snap overlay exists for nearest-snap queries only. It must not define note rendering, note edge lookup, or canonical hit-testing geometry.
- Duration rendering should be implemented as address-span projection using lattice indices, not by multiplying one width value. The intended model is:

```ts
const startIndex = lattice.byAddressKey.get(addressKey(startAddress));
const endIndex = startIndex + duration;
const startX = lattice.continuous[startIndex].x;
const endX = lattice.continuous[endIndex].x;
const width = endX - startX;
```

- The exact helper API names may vary, but the story expects this capability set:
  - address to stable key
  - address to continuous index
  - continuous index to address
  - address to x
  - x to nearest valid address or nearest snapped address
  - start address plus duration to end address or end index
  - start address plus duration to `{ startX, endX, width }`

- The story intentionally treats tuplets as first-class replacement regions in the projection model. Any implementation that flattens tuplet-local ticks into host-tick floats, or that treats tuplets as a workaround layered over a host-only lattice, does not satisfy this technical direction.

### Expected Helper API Surface

- The exact names may vary, but the implementation is expected to expose a helper surface close to the following:

```ts
type AddressKey = string;

type PixelSpan = {
  startX: number;
  endX: number;
  width: number;
};

type AddressLookupDirection = "nearest" | "floor" | "ceil";

type SequencerAddressLattice = {
  continuous: AxisPoint[];
  byAddressKey: Map<AddressKey, number>;
};

type SequencerSnapOverlay = {
  ticksPerSnap: number;
  points: SnapPoint[];
};

function addressKey(address: AxisAddress): AddressKey;
function getIndexForAddress(
  lattice: SequencerAddressLattice,
  address: AxisAddress,
): number | null;
function getAddressAtIndex(
  lattice: SequencerAddressLattice,
  index: number,
): AxisAddress | null;
function getXForAddress(
  lattice: SequencerAddressLattice,
  address: AxisAddress,
): number | null;
function findContinuousIndexForX(
  lattice: SequencerAddressLattice,
  x: number,
  direction: AddressLookupDirection,
): number | null;
function findAddressForX(
  lattice: SequencerAddressLattice,
  x: number,
  direction: AddressLookupDirection,
): AxisAddress | null;
function advanceIndex(
  lattice: SequencerAddressLattice,
  startIndex: number,
  duration: number,
): number | null;
function getSpanForAddressDuration(
  lattice: SequencerAddressLattice,
  start: AxisAddress,
  duration: number,
): PixelSpan | null;
function buildSnapOverlay(
  lattice: SequencerAddressLattice,
  ticksPerSnap: number,
): SequencerSnapOverlay;
function findSnappedAddressForX(
  overlay: SequencerSnapOverlay,
  x: number,
  direction: AddressLookupDirection,
): AxisAddress | null;
```

- The implementation does not need to expose exactly these exported function names, but it must support these operations without forcing calling code back into width multiplication or host-tick translation.
- `getSpanForAddressDuration(...)` is the critical primitive for rendering and resize handles. The track UI should ask for a span from a start address and duration rather than calculating width from a single tick width.
- `findAddressForX(...)` and `findSnappedAddressForX(...)` are distinct on purpose:
  - canonical address lookup answers “what boundary is at or nearest this x?”
  - snapped lookup answers “what valid snapped boundary should this gesture commit to?”
- `advanceIndex(...)` or its equivalent must move through the canonical continuous lattice one semantic tick at a time in the local owning context. It must not convert through host ticks or float ratios.

### Implementation Invariants

- `continuous` must be strictly ordered by increasing x.
- Every address key in `byAddressKey` must resolve to exactly one canonical index.
- No canonical index may represent both a host address and a tuplet address at the same horizontal slot.
- Entering a tuplet region in the canonical lattice means subsequent indices represent tuplet-local addresses until the tuplet region ends.
- Leaving a tuplet region in the canonical lattice means subsequent indices resume host-owned addresses after the replaced host span.
- A tone that crosses a boundary must be rendered from one start address and one duration against the canonical lattice. It must not be rendered as if the full tone used one uniform local width.
- The active snap overlay may be rebuilt when tool state changes. The canonical lattice must not be rebuilt solely because snap duration changed.
- Rendering, hit-testing, and gesture snapping must all be able to trace their result back to the same canonical lattice, even if snapping uses a derived sparse overlay.

### Illustrative Lattice Construction Example

- The following example is illustrative, not prescriptive. It exists to clarify the intended shape of the canonical lattice, not to require a specific loop structure, storage strategy, or exact helper decomposition.

```ts
const continuous: AxisPoint[] = [];

for (const region of orderedRegions) {
  if (region.type === "host") {
    for (let tick = region.startTick; tick <= region.endTick; tick += 1) {
      continuous.push({
        address: { tuplet_id: null, tick },
        x: region.startX + (tick - region.startTick) * region.tickWidth,
      });
    }
    continue;
  }

  for (let tick = 0; tick <= region.localLengthTicks; tick += 1) {
    continuous.push({
      address: { tuplet_id: region.tupletId, tick },
      x: region.startX + tick * region.localTickWidth,
    });
  }
}

const byAddressKey = new Map(
  continuous.map((point, index) => [addressKey(point.address), index]),
);
```

- The key intent is the ordering, not the exact code shape:
  - host boundaries appear before the tuplet starts
  - the tuplet region contributes only tuplet-local addresses while it replaces that span
  - host boundaries resume after the replaced span ends
- A production implementation may build the same result via region iterators, generators, precomputed segments, or another equivalent approach.
- If performance requires a more compact internal representation, that is acceptable as long as the exposed semantics still behave exactly like this canonical ordered boundary lattice.

### Architecture Compliance

- Rust remains the source of truth for accepted tone edits. Frontend drag math may preview or clamp, but final validity must be enforced in backend-owned mutation code.
- Tuplets are local metric frames with child event streams. Validation must respect the tuplet's local frame rules rather than flattening everything to host-only heuristics.
- `tone_id` remains authoritative identity. `tuplet_id` is lookup and validation context, not a second identity model.
- The canonical frontend coordinate model is an address lattice, not a host-tick ruler with tuplet exceptions. Tuplets must behave like first-class replacement regions in the projection model.
- Crossing a tuplet boundary does not itself force storage splitting. Rendering and playback remain responsible for derived boundary fragments.
- Host-track tones remain owned by where they start, even when a resize extends them through a tuplet span. Tuplet-owned tones remain child entries while their start stays inside the tuplet. When the start crosses the boundary, ownership must move to the new owning context without splitting the tone into multiple stored events.
- Snap duration is literal rather than context-specific in meaning. The same chosen written duration must project through either host or tuplet timing depending on where the segment is being edited.
- Active snap duration must not define canonical geometry. It may define a derived snapping overlay only.
- `ui.flow_id` remains the one authoritative active-flow slot across score, Sequence, miniature piano rolls, and transport-facing consumers.

### Worked Example

- Example: a tone starts on the second tuplet eighth of an eighth-note triplet and is given a duration of four eighths.
- Storage: because the tone starts inside the tuplet, it is stored as one tuplet-owned child tone, not split into separate persisted host and tuplet tones.
- Duration meaning: the four eighths remain literal written duration intent. Two of those eighths fall inside the remaining tuplet span and the remaining two continue in ordinary track time after the tuplet boundary.
- Ownership: the tone stays tuplet-owned because its start remains inside the tuplet. If a later edit moved the start outside the tuplet, ownership would transfer to the host track while the tone still remained one stored event.
- Render/playback result: the boundary split is derived rather than persisted. The user should experience one continuous tone that begins at the second tuplet eighth, extends through the rest of the tuplet, and then continues for two ordinary eighths beyond the tuplet boundary.
- Sequencer expectation: no extra tuplet-specific visual model is required beyond keeping the tuplet boundary and inner beats legible while the crossing tone remains visibly continuous across that boundary.

### Library / Framework Requirements

- Frontend work stays in the current React 19 + TypeScript + Tauri invoke stack.
- Backend mutation authority remains Rust-owned state behind existing Tauri commands.
- No new dependency is expected for this story.

### File Structure Requirements

- Primary frontend seams:
  - `src/windows/main/components/sequencer-track.tsx`
  - `src/windows/main/components/sequencer-track-tuplet.tsx`
  - `src/windows/main/components/sequencer-tuplet-utils.ts`
  - `src/services/actions.ts`
  - `src/services/sequencer-utils.ts` only if click-target plumbing changes
- Primary backend seams:
  - `src-tauri/src/score/entries/tone.rs`
  - `src-tauri/src/score/tracks.rs`
  - `src-tauri/src/score/entries/tuplet.rs` only if helper methods or tests need extension
- Existing tests to extend before adding new harnesses:
  - `src/windows/main/components/sequencer-track.test.tsx`
  - `src/services/sequencer-utils.test.ts`
  - backend tests in `src-tauri/src/score/entries/tone.rs`
  - backend tests in `src-tauri/src/score/tracks.rs`

### Testing Requirements

- Verify dragging a tuplet-owned tone horizontally within supported bounds updates its timing through exact tuplet-local rules.
- Verify vertical drag re-pitches a tuplet-owned tone using the same pitch-targeting rules and optional audition behavior as ordinary tones.
- Verify right-edge and left-edge resize of a tuplet-owned tone stay within supported local bounds unless the accepted outcome explicitly crosses the tuplet boundary under the one-stored-event rule.
- Verify crossing-boundary resize preserves one stored tone and leaves boundary splitting to derived render/playback behavior.
- Verify dragging a tone's start across the tuplet boundary moves ownership to the new start context in both directions while preserving one stored event.
- Verify literal snap duration remains stable across the boundary, using tuplet-local values inside the tuplet and track values outside it.
- Verify canonical address projection and duration-span projection remain correct when a rendered tone crosses between host and tuplet regions.
- Verify the current snap duration does not alter canonical note rendering or existing note edge lookup.
- Verify unsupported overlap, invalid local-frame outcomes, and nested-tuplet-requiring edits are rejected without approximate timing fallbacks.
- Verify the tuplet region, slot markers, hit targeting, and selection remain stable after rejected edits.
- Verify by-tick and by-id indexes remain consistent after accepted tuplet drag and resize operations.
- Verify tuplet containers themselves are not resizable through these gestures.
- Verify no multi-tone editing path is added or enabled.
- Undo/redo is not an acceptance requirement for this story because the app does not yet provide that infrastructure.

### Established Code Patterns To Follow

- Extend shared seams before adding new ones. Recent tuplet work in Stories 2.2 and 2.3 reused existing tone mutation and sequencer-targeting seams with explicit `tuplet_id` context.
- Treat tuplet membership as nested storage context, not a product-level alternate edit mode.
- Keep validation in Rust even if the frontend also clamps obvious invalid drags.
- Keep the visual tuplet region component focused on presentation and stability. Do not shift score truth or gesture state into the UI layer.
- Keep Story 2.4 tightly scoped to single-tone drag editing. Do not pull container delete, slice policy, tuplet-container resizing, or generalized multi-note editing into this story unless the epic source is revised.
- Do not continue extending a boundary-fix cursor-translation model as the primary solution. The corrected solution is a single authoritative address lattice shared by rendering, hit-testing, and snap lookup.

### Previous Story Intelligence

- Story 2.1 solved reliable targeting for tuplet members and clarified that click resolution should return explicit `tuplet_id` context when a tone lives inside a tuplet.
- Story 2.2 solved tuplet-owned pitch parity by threading `tuplet_id` through shared tone pitch seams instead of adding a tuplet-only mutation API.
- Story 2.3 solved tuplet slot add/remove behavior by reusing the shared sequencer/action/Rust seams and preserving empty-slot rendering as visual context rather than persisted rest data.
- The same pattern should govern Story 2.4: do not invent a separate tuplet gesture model when the existing select-tool handlers and tone commands can be refined.

### Git Intelligence Summary

- Recent relevant commits:
  - `96cdae7d feat(2.3): basic tone editing`
  - `802f5e64 fix: no tuple highlights`
  - `6cd8bef8 feat(2.2): finish tuplet pitch editing parity`
  - `c8ea0449 feat(2.1): tuple tone selection info`
- The recent pattern is incremental seam extension in the sequencer and tone mutation layers, backed by focused regressions rather than new subsystem creation.
- Current code inspection shows the frontend already has generic drag handlers and the backend already has shared tone movement APIs, but tuplet-owned select-tool dragging is currently blocked early and backend move/resize validation is still host-track-centric.

### Documentation Impact

- This story may change documented tuplet editing semantics and should trigger a documentation impact review.
- Review these docs during implementation and update them if the shipped contract changes materially:
  - `docs/architecture.md`
  - `docs/development-guide.md`
  - `docs/project-context.md`
  - `.github/copilot-instructions.md` if any durable guardrail changes
  - `tests/fixtures/README.md` if new tuplets-related visual regression fixtures are introduced
- If implementation only fulfills already-documented behavior, record that no documentation update was required in the completion notes rather than skipping the review silently.

### Project Structure Notes

- The prior in-progress implementation added cursor-based projection helpers and debugging to compensate for the current host-tick-based snap model. That work is now considered exploratory and should not constrain the corrected implementation.
- `SequencerTrackTuplet` already renders a bounded region with slot markers using `data-tuplet-id`; that existing structure should remain the hit-target and orientation foundation during drag work.
- Main implementation risks:
  - allowing rendering and hit-testing to diverge because they query different address or snap views
  - retaining host-ruler pixel math as the true snapping authority and then layering tuplet exceptions on top
  - letting current snap duration leak into canonical geometry rather than a derived snap overlay
  - corrupting ownership or indexes when a start drag crosses the tuplet boundary
  - accidentally broadening scope into multi-note moves or Story 2.5 delete/slice rules

### Open Questions / Assumptions

- Clarified: ownership transfer is required in both directions when a tone start crosses the tuplet boundary.
- Clarified: snap duration is literal across the boundary. Inside the tuplet, snapping uses tuplet-local equivalents of the active snap value; outside it uses track timing with the same written duration meaning.
- Clarified: canonical sequencer geometry is independent of current snap duration. If a sparse snap view is introduced, it is derived from the canonical address lattice rather than replacing it.
- Clarified: Story 2.4 is single-tone only. No grouped movement or resize behavior should be added.
- Clarified: tuplets themselves are not resizable.
- Assumption retained: pitch drag should behave identically to ordinary tones because Story 2.2 already established pitch-parity through shared seams.
- Assumption retained: unsupported edits should reuse the current local rejection behavior with no new tuplet-specific message chrome.
- Clarified: the intended visual result for tones whose duration extends in or out of a tuplet is the existing architecture rule of one stored tone plus derived boundary split in render/playback, with the tuplet boundary and inner beats remaining legible while the tone reads as continuous.
- Assumption retained: the current story artifact completion notes and prior implementation file list were based on a reverted implementation and should not be treated as shipped work.

### References

- Source: `_bmad-output/planning-artifacts/epics.md` — Epic 2 / Story 2.4 acceptance criteria and Story 2.5 boundary
- Source: `_bmad-output/planning-artifacts/prd.md` — editing semantics, exact timing, one-stored-event rule, boundary crossing, and non-nestable MVP behavior
- Source: `_bmad-output/planning-artifacts/architecture.md` — local metric frame model, ownership-by-start rules, derived boundary splitting, and sequencer UX contract
- Source: `_bmad-output/planning-artifacts/ux-design-specification.md` — sequencer-first tuplet editing, bounded-region presentation, slot-marker visibility, and rejection/recovery expectations
- Source: `docs/project-context.md` — Rust-owned truth, shared action seams, `ui.flow_id` guardrail, and score-render architecture
- Source: `_bmad-output/implementation-artifacts/2-3-add-and-remove-notes-within-tuplet-slots.md` — previous story seam-reuse pattern and scope discipline
- Source: recent code inspection of `src/windows/main/components/sequencer-track.tsx`, `src/windows/main/components/sequencer-track-tuplet.tsx`, `src/windows/main/components/sequencer-tuplet-utils.ts`, `src/services/actions.ts`, `src/services/sequencer-utils.ts`, `src-tauri/src/score/entries/tone.rs`, and `src-tauri/src/score/tracks.rs`

## Dev Agent Record

### Agent Model Used

GPT-5.4

### Debug Log References

- Reviewed Epic 2 Story 2.4 acceptance criteria and adjacent story boundaries in `_bmad-output/planning-artifacts/epics.md`.
- Reviewed PRD guidance for exact timing, boundary-crossing edits, one-stored-event ownership, and non-nestable MVP rules in `_bmad-output/planning-artifacts/prd.md`.
- Reviewed architecture guidance for local metric frames, ownership-by-start, derived boundary splits, and sequencer obligations in `_bmad-output/planning-artifacts/architecture.md`.
- Reviewed UX guidance for sequencer-first tuplet editing, bounded-region presentation, slot markers, and rejection stability in `_bmad-output/planning-artifacts/ux-design-specification.md`.
- Reviewed Story 2.3 implementation guidance to preserve seam-reuse and scope discipline.
- Inspected current drag/edit seams in `src/windows/main/components/sequencer-track.tsx`, `src/windows/main/components/sequencer-track-tuplet.tsx`, `src/windows/main/components/sequencer-tuplet-utils.ts`, `src/services/actions.ts`, `src/services/sequencer-utils.ts`, `src-tauri/src/score/entries/tone.rs`, and `src-tauri/src/score/tracks.rs`.
- Reviewed recent git history to align the story with the current incremental tuplet implementation pattern.
- Captured and then resolved product decisions around ownership transfer, literal snap duration, non-resizable tuplets, single-tone-only editing, and the intended visual result for boundary-crossing durations.

### Current Reset Notes

- Story 2.4 remains intentionally scoped to single-tone drag editing inside the existing sequencer-first workflow.
- Story 2.5 retains explicit delete and slice policy work.
- 2026-03-17 correct-course update: the first implementation path is being discarded because boundary-fix cursor translation is not the right architectural center for tuplet-aware gesture editing.
- The corrected Story 2.4 direction is an authoritative sequencer address lattice shared by tone rendering, gesture hit-testing, and snap lookup, while the host tick bar remains visual-only and the tuplet overlay remains the cue for morphed local spacing.
- Prior completion notes and file lists reflected reverted exploratory work and should not be treated as shipped implementation.

### File List

- `_bmad-output/implementation-artifacts/2-4-reposition-or-resize-tuplet-notes-within-supported-bounds.md` — re-baselined the story around the address-lattice model after reverting the obsolete implementation direction

### Change Log

- 2026-03-16: Created Story 2.4 implementation artifact with scope boundaries, shared-seam guidance, backend validation requirements, testing expectations, documentation impact tasks, and surfaced open questions.
- 2026-03-17: Corrected Story 2.4 implementation direction after the first implementation path proved architecturally unsound for tuplet-aware projection and hit-testing. Story status returned to in-progress and guidance first moved to an axis-map approach.
- 2026-03-17: Re-based the technical direction again after validating that width-centric axis-map math still treated tuplets as an exception layer. Story guidance now requires an authoritative sequencer address lattice whose canonical geometry is independent of active snap.
