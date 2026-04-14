## Deferred from: code review of 8-2-render-and-select-playing-technique-markings-above-the-stave.md (2026-04-09)

- Same-tick playing-technique siblings are not individually selectable because coexistence currently reuses one canonical anchor and overlapping hit targets. Deferred for now because this story is proving the concept; proper text collision and coexistence handling will be addressed later as a larger holistic change.

## Deferred from: code review of spec-halion-controller-init-contract-variants.md (2026-04-11)

- The maintainer load/init probe currently writes its result back through the shared `editor_host_validation` payload even for normally supported plugins. That is acceptable for the blocked HALion-family investigation path, but a follow-up should separate experimental probe evidence from canonical ordinary validation before the maintainer probe is used broadly on supported bundles.
