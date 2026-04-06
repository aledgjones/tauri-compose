## Deferred from: one-shot fix of story-6.3-dnd-alignment (2026-04-06)

- Expression-map selected-rule tracking still relies on object identity across a backend/store refresh; after refreshed rule objects arrive, selection can fall back to a stale pre-reorder index and point the editor at the wrong rule.
- Expression-map ID rekey still leaves the rest of the editor writable while the rekey request is in flight, so follow-up autosaves can race the old and new IDs.

## Deferred from: code review of 6-3-create-one-dedicated-expression-map-editor-window.md (2026-04-06)

- Legacy expression-map library payloads that lack the new nested ids or metadata fields still need a migration path before release; deferred for now because no legacy expression maps exist while the feature remains development-only.
