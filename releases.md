## Release Notes for v2.1.0

### 🔧 Chores

- chore: clean up unused func

### 📝 Other

- Merge pull request #5 from aledgjones/feat/triplets

## Release Notes for v2.0.0

### 🏗️ Features

- feat: performance baselines
- feat: unify stem directions inside tuplets
- feat: cleanup mcp onboarding
- feat(SC-175): draw tuplets on mini piano roll
- feat: gould style spacing across rhythms
- feat(4.4): keep MVP playback scoped to active flow
- feat(4.3): preserve host-only playback behavior
- feat(4.2): finalize tuplet playback ordering
- feat(4.1): tuple playback
- feat(3.2, 3.3, 3.4): triplet notation
- feat(3.2): splttiing
- feat(2.5): tuplet aware slice and delete
- feat(2.3): basic tone editing
- feat(2.2): finish tuplet pitch editing parity
- feat(2.1): tuple tone selection info
- feat: epic 1 retro
- feat(1.7): validate tuplet file workflow contract
- feat: upgrade bmad prep
- feat(1.6): tuplet file loading regression tests
- feat(1.5): reconcile tuplets over existing material
- feat(1.4): add tuplet note entry in sequencer
- feat(1.3a): tuplet entries structures non-recursive
- feat: course correct
- feat(1.3): description
- feat(1.2): description
- feat(1.2): add tuplet popover creation flow
- feat(1.1): init tuplets
- feat(3.5): set current performance baseline warning
- feat(3.3): isolate recalculation to affected flow
- feat(2.1): flow rendering debug info
- feat(2.5): preserve active flow across score and sequence
- feat(2.4): verify multi-flow popover anchors
- feat(2.3): simplify selection clearing for deletes
- feat(2.1): hit tet validation
- feat(1.6): render all visible flows on one surface
- feat(1.4): refine adjacent flow score coherence
- feat(1.4): add per-flow page backgrounds
- feat(1.3): renderer scrolling
- feat: render two flows on a shared score surface
- feat(bmad): PRD
- feat: leading quarter rest in compound
- feat: ensure writtable full bar notes
- feat(SC-171): compound time signature initial dumb pass
- feat: tighten glyph rendering, and debug
- feat: auto select newly created entries

### 🐞 Fixes

- fix: tighten tuplet hit targets
- fix: sample trimming
- fix: mcp documentation
- fix: tuplet model description in mcp server
- fix: formatting
- fix: app hang due to state access.
- fix: mcp alignment
- fix: tuplet timings encoded in agent instructions
- fix: snap expectations.
- fix: no tuple highlights
- fix: tweak tuple background colour
- fix: tuple popover trigger
- fix: epic cleanup
- fix: more deterministic empty bar spacing
- fix: 6/8 test cases
- fix: only apply some rules at eigth resolution
- fix: unify test script names
- fix: sync with main
- fix: 3/4 split correct when quarter offbeat.
- fix: small labeling tweak
- fix: tempo styling
- fix: tempo offset
- fix: better empty states
- fix: disable hotkeys responding when inside a text input
- fix: update c/cutc text
- fix: tweak tempo standard vetical offset
- fix(release): revert migrations next to template

### 🔧 Chores

- chore: epic 5 retro
- chore: epic 5 stories
- chore: epic 4 retro
- chore: epic 3 retro
- chore: update stories to be more procise
- chore: prep epic 3
- chore: epic 2 retro
- chore(2.5): mark story done
- chore: update story to follow arch guidlines
- chore(2.4): story
- chore: upgrade bmad
- chore: planning
- chore: remove accidental reapply of file
- chore: formatting
- chore: archive stale docs
- chore: tuplet prep
- chore: complete prd
- chore: cleanup
- chore: cleanup sprint status
- chore: epic 4 retro
- chore: epic 4 prep
- chore: refactor render trigger API
- chore(3.1): prep
- chore: epic 2 retro
- chore: epic 1 retro
- chore(story): close rejected flow label direction
- chore(plan): align multi-flow roadmap and story scope
- chore: sprint prep
- chore: epics docs
- chore: architecture docs
- chore: ux docs setup
- chore: PRD review
- chore: fix formatting
- chore: refactor tests into unified scenario structure

### 📝 Other

- Refine forward tuplet spacing successor logic
- docs(epic-5): finalize tuplet docs and guardrails
- test(epic-5): harden tuplet regression coverage
- Complete Story 2.4 tuplet drag and snap work
- Merge branch 'main' into feat/triplets
- Expose Compose automation as generated MCP tools
- Merge branch 'main' into feat/triplets
- Add app-hosted MCP automation server
- Merge pull request #4 from aledgjones/feat/multi-flow-rendering
- docs(4.4): record offline-first boundary audit
- test(4.3): preserve active-flow playback behavior
- test(4.2): guard desktop workflow shell behavior
- test(4.1): guard document round-trip behavior
- test(3.2): add multi-flow notation safety regressions
- test(2.2): cover multi-flow selection consumers
- Merge pull request #3 from aledgjones/feat/SC-171-compound-time-signatures

## Release Notes for v1.25.2

### 🏗️ Features

- feat: unify properties panel structure
- feat: all entry selection
- feat: quick spec selection cleanup

### 🐞 Fixes

- fix: small tweaks to hit targets
- fix(release): skip prep manifest in reseed stage

### 🔧 Chores

- chore: bump versions

## Release Notes for v1.25.1

### 🏗️ Features

- feat: release script correction
- feat: new release flow
- feat: release and migrations spec

### 🐞 Fixes

- fix: formatting

## Release Notes for v1.25.0

### 🏗️ Features

- feat: tempo properties

## Release Notes for v1.24.0

### 🏗️ Features

- feat: absolute tempo playback
- feat: tempo markings
- feat: test fixtures
- feat: test fixtures
- feat: container file format
- feat: file format spec
- feat: fixtures tech spec
- feat: fixture files
- feat: fixtures
- feat: add debugger score fixtures
- feat: edge case analysis

### 🐞 Fixes

- fix: remove dead spec
- fix: incorrect splitting at for -- quarter, quareter tied to eigth and eigth rest.
- fix: formatting descripencies
- fix: add missing key prop to SelectByImage; silence console.log in actions tests
- fix: remove nested button in PlayerPanelItem, add onPointerDown to IconButton

### 🔧 Chores

- chore: properties panel quick spec
- chore: remove dead spec
- chore: cleanup fixture code
- chore: test spec
- chore: ratchet rust coverage thresholds
- chore: test plan
- chore: extract coverage summary into scripts/coverage-summary

### 📝 Other

- docs: add tech spec for binary .soloc project file format
- docs: add tech spec for binary .soloc project file format
- fix signature spacing and add cancellation fixtures
- test: expand notation pattern regression coverage
- test: harden written duration regression coverage
- test: cover renderer pan and bounds
- refactor: share sampler sync and player setup logic
- test: expand score flow and instrument coverage
- test(rust): Phase 4 — barline/beam/ledger/brackets/ties tests + extract beam_angle/stem_tweak helpers
- test(rust): Phase 3 — draw_notehead/rest/articulations tests + tone medium command extractions
- test(rust): Phase 2 complete — draw tests + tone impl extraction + clippy fixes
- test(rust): Phase 1 complete — all 7 trivial draw files tested
- test(rust): Phase 1 draw test infrastructure + stem/clef tests
- test(rust): expand coverage across parse and score modules
- test: expand FE coverage to 80%+ on all metrics
- test: remove redundant mockImplementation from consoleSpy (global setup handles silencing)
- test: fix act() warning in instrument-picker tests

## Release Notes for v1.23.3

### 🏗️ Features

- feat: test coverage expansion docs

### 🐞 Fixes

- fix: further test coverage improvements
- fix: coverage summary script

### 🔧 Chores

- chore: test coverage improvements
- chore: bump test coverage
- chore: code review test coverage
- chore: rust test coverage
- chore: ts test suite
- chore: don't include coverage reports in git history

### 📝 Other

- test: add Rust coverage scripts and update TESTING.md coverage docs
- test: expand FE coverage with 24 new test files, raise thresholds
- test: expand FE coverage to 53% and bump thresholds

## Release Notes for v1.23.2

### 🏗️ Features

- feat(tooling): add testing foundation, modernize ESLint, and add pre-commit hooks
- feat: testing/tooling spec
- feat: project overview
- feat: BMad

### 🔧 Chores

- chore: revert Phase 4 E2E work — tauri-driver v2 is Linux-only

## Release Notes for v1.23.1

### 🏗️ Features

- feat: style tweaks left panel

### 🐞 Fixes

- fix: spacing tweaks for accidentals. tighten spacing and allow accidentals to eat inot more of the note spacing.

## Release Notes for v1.23.0

### 🏗️ Features

- feat: tone selection panel
- feat: rendering articulations more accurately
- feat: accidentals
- feat: articulations and accidental panel

## Release Notes for v1.22.1

### 🏗️ Features

- feat: better release process

## Release Notes for v1.22.0

### 🏗️ Features

- feat: articulations buttons
- feat: copilot instructions update
- feat: organise buttons into folder
- feat: instructions file for agents
- feat: setup articulation iterations
- feat: staccatissimo rendering
- feat: basic articulation rendering

### 🐞 Fixes

- fix: unused imports

### 🔧 Chores

- chore: add rule to instructions, no short var names.

## Release Notes for v1.21.0

### 🏗️ Features

- feat: unify articulations on tone creation
- feat: unify articulations on tone left handle adjustment
- feat: unify articulations on end of drag
- feat: moving tones unifies articulations

### 🐞 Fixes

- fix: align types
- fix: refine release process

### 🔧 Chores

- chore: remove prompt config

## Release Notes for v1.20.7

### 🐞 Fixes

- fix: open correct link to release

## Release Notes for v1.20.6

### 🐞 Fixes

- fix: release scripts

## Release Notes for v1.20.5

### 🏗️ Features

- feat: unify articulations in chords
- feat: update status
- feat: prompts update
- feat: mcp jira workflow

## Release Notes for v1.20.4

### 🏗️ Features

- feat: toggle articulations

## Release Notes for v1.20.3

### 🐞 Fixes

- fix(SC-000): release supports ids

## Release Notes for v1.20.2

### 🏗️ Features

- feat: release updates

## Release Notes for v1.20.1

### 🏗️ Features

- feat: initial articulation concept
- feat: default sustain articulation
- feat: initial flute 1 pass
- feat: gong

### 🐞 Fixes

- fix: cleanup unused exporessions
- fix: cleaner gradient

### 🔧 Chores

- chore: clippy fixes
- chore: cleanup legacy instrument

### 📝 Other

- feat(SC-141): align articulation enums and implement keyboard shortcuts for articulations

## Release Notes for v1.20.0

### 🏗️ Features

- feat: complete drum kit
- feat: cymbals
- feat: more drum kit articulations
- feat: drum kit partial
- feat: bongo
- feat: bass drum
- feat: violin 2, tubular bells
- feat: violin 1
- feat: violin samples
- feat: harpsichord
- feat: flac and manifest
- feat: viola partial
- feat: cello
- feat: vorbis samples
- feat: refresh audio devices
- feat: percussion ai pass
- feat: ranges
- feat: streaming samples
- feat: articulation ordering
- feat: binary samples
- feat: clarinet
- feat: bassoon
- feat: flute
- feat: simpler instrument structure
- feat: cello
- feat: basic auditioning
- feat: reduce piano size
- feat: cleanup instrument and integrity check
- feat: init sampler window
- feat: sampler state
- feat: active only filters
- feat: scvi search
- feat: scvi instruments
- feat: flac and mp3 instruemnt encoding
- feat: piano
- feat: add ids to samplers with lite variant
- feat: plugin loading info with correct meta

### 🐞 Fixes

- fix: formatting
- fix: duration fixes
- fix: bump piano version
- fix: plugin tweaks
- fix: violin instrument
- fix: covers
- fix: perf timing style tweaks
- fix: correct paths in generated scvi instruments
- fix: meta structure piano
- fix: plugin types TS
- fix: build issues

### 📝 Other

- faet: muse samples

## Release Notes for v1.19.7

### 🏗️ Features

- feat: generate wavpack and mp3 lite version s of the scvi instruments.
- feat: wavack compression

## Release Notes for v1.19.6

### 🐞 Fixes

- fix: script organisation
- fix: instrument folder structure
- fix: cleanup temp folders

## Release Notes for v1.19.5

### 🐞 Fixes

- fix: ensure proper spacing in release notes formatting

## Release Notes for v1.19.4

### 🐞 Fixes

- fix: correct release notes formatting to use single blank lines

## Release Notes for v1.19.3

### 🏗️ Features

- feat: add git workspace cleanliness check to release script

## Release Notes for v1.19.2

### 🐞 Fixes

- fix: release script cargo.lock wait

## Release Notes for v1.19.1

### 🏗️ Features

- feat: init sampler
- feat: setup vst loading
- feat: device and sample rate selection
- feat: use device id identification
- feat: seperate concerns better
- feat: backend
- feat: strip old playback system
- feat: unify state patterns
- feat: rest selection indicator
- feat: popups
- feat: popover positioning and interactions
- feat: show popover
- feat: selection
- feat: reorganise screens
- feat: init scroll drag with x,y
- feat: dotted notes at the start of a unit
- feat: 3/4 first pass
- feat: basic 3/4 rendering
- feat: dynamic layouts/flows
- feat: 2/4 signatures
- feat: correct dotted minim splitting
- feat: better timing debgginng
- feat: simplify code
- feat: simplify scrollbars
- feat: standard shortcuts
- feat: simplify context
- feat: split text files
- feat: split state files
- feat: preferences
- feat: audition tones
- feat: better selection and audition
- feat: nicer background
- feat: seperate padding for better organisation
- feat: delete tone keybord shortcut
- feat: hit targets
- feat: keyboard shortcuts
- feat: add track id in selection for easy entry manipulation
- feat: restructure screens
- feat: ledger line tweaks for accidentals
- feat: file handling
- feat: better readme
- feat: zoom
- feat: padding and scrolling
- feat: scrolling
- feat: scroll bars
- feat: simpler scaling system
- feat: migrations
- feat: fixed subdivisions
- feat: ties tweaks
- feat: sub beat splitting
- feat: debug rendering
- feat: ledger fixes and note metrics

### 🐞 Fixes

- fix: remove static asset loading
- fix: types
- fix: minor cleanup
- fix: naming
- fix: seperate selection types
- fix: cumulative barline
- fix: allow spanning mid point for dotted notes at end of beat
- fix: tighter barlin spacing with time/key sigs
- fix: properly clear state on resets and loads
- fix: dotted quarter spans
- fix: beaming
- fix: formatting
- fix: debug layout
- fix: transport
- fix: DRY
- fix: scroll bar rendering simplifications
- fix: rest splitting
- fix: beam lengths
- fix: reduce duplication in beam drawing
- fix: stem accuracy
- fix: bar padding on first bar too
- fix: match dorico colors
- fix: types
- fix: lint errors
- fix: imports
- fix: ts warnings
- fix: eslint
- fix: emit preference state
- fix: simplify tone info
- fix: render on different displays
- fix: metrics
- fix: mut
- fix: better naming
- fix: recents
- fix: menu recent name change
- fix: trailing rests
- fix: remove stupid comments
- fix: correct delta
- fix: formatting
- fix: correct scale factor on external monitor
- fix: time sig spacing
- fix: extra spacing tweaks
- fix: text input caret jumping
- fix: unify handlers and prevent overscroll
- fix: min space tweaks
- fix: debugger styling
- fix: formatting
- fix: include breathing space in accidentals width calc
- fix: exact accidental spacing
- fix: accidentals with pre-shunts
- fix: playhead z index correction

### 🔧 Chores

- chore: remove samples
- chore: bump versions
- chore: better renderer organisation
- chore: bump deps
- chore: update docs
- chore: simpler debugging.
- chore: bump deps
- chore: bump versions

### 📝 Other

- Merge pull request #2 from aledgjones/feat/fixed-subdevisions

## Release Notes for v0.19.0

### 🏗️ Features

- feat: cleanup piano roll ui
- feat: simplify patches
- feat: audio assets
- feat: unified app bar
- feat: disabled instruemnts
- feat: basic sequencer screen
- feat: slice tones
- feat: tone selection

### 🐞 Fixes

- fix: only render hen render region is mounted
- fix: linting
- fix: shade areas in seuencer overview
- fix: render order
- fix: anti aliasing

## Release Notes for v0.18.0

### 🏗️ Features

- feat: style tweaks
- feat: more ui-refinements
- feat: style tweaks
- feat: refine layout
- feat: tone adjustments
- feat: dragging across ticks
- feat: drag pitches
- feat: tone selection
- feat: tone selection
- feat: tone names

### 🐞 Fixes

- fix: chevron icons
- fix: fractional at eigth boundry
- fix: quick tweaks to padding
- fix: new score resetting
- fix: access errors

## Release Notes for v0.17.0

### 🏗️ Features

- feat: oom
- feat: native renderer
- feat: renderer structure
- feat: restructure px per mm

### 🐞 Fixes

- fix: prettier warnings
- fix: warnings tailwind
- fix: proper clipping
- fix: singleton renderer
- fix: bounds
- fix: cleanup
- fix: cleanup renderer view
- fix: minor formatting
- fix: formatting
- fix: cleanup

## Release Notes for v0.16.0

### 🏗️ Features

- feat: upgrade deps
- feat: settings
- feat: init refactor
- feat: zoom init
- feat: panels
- feat: mixer colouring
- feat: compress files
- feat: file actions
- feat: initial tie setup
- feat: add harpsichord
- feat: fractionals
- feat: further perf optimisations
- feat: split sampler state
- feat: mixer
- feat: setup bottom panel tabs
- feat: playback
- feat: sampler
- feat: sampler code
- feat: cure rendering
- feat: tails
- feat: parrallel
- feat: perf tracking and font cache
- feat: minor cleanup

### 🐞 Fixes

- fix: grids
- fix: menu style tweaks
- fix: layout sequencer
- fix: linting
- fix: style tweaks
- fix: channel tweaks
- fix: lint warns
- fix: deps
- fix: unused var
- fix: instrument ordering
- fix: whole note rendering
- fix: formatter run
- fix: deserialse
- fix: ts warnings
- fix: play sample at correct time!
- fix: json formatting
- fix: formatting
- fix: ledger lines
- fix: clippy bippy
- fix: bump deps and fix issues

### 🪚 Maintainence

- chore: bump versions
- chore: bump deps

## Release Notes for v0.15.0

### 🏗️ Features

- feat: note spacing

### 🪚 Maintainence

- chore: refactor metrics for consistency

## Release Notes for v0.14.0

### 🏗️ Features

- feat: draw dots
- feat: draw guide beams

### 🐞 Fixes

- fix: tones on central line should default to up stem

### 🪚 Maintainence

- chore: better commenting

## Release Notes for v0.13.1

### 🐞 Fixes

- fix: beam stemming

### 🪚 Maintainence

- chore: remove debugging
- chore: bump

## Release Notes for v0.13.0

### 🏗️ Features

- feat: perfrmance restructure of render
- feat: init stems -- broken!

## Release Notes for v0.12.0

### 🏗️ Features

- feat: render noteheads basic

## Release Notes for v0.11.1

### 🐞 Fixes

- fix: refactor for performance

### 🪚 Maintainence

- chore: performance refactor of barlines
- chore: bump versions
- chore: cleanup comments

## Release Notes for v0.11.0

### 🏗️ Features

- feat: draw accidentals

### 🪚 Maintainence

- chore: cleanup debug code
- chore: cleanup debug code

## Release Notes for v0.10.0

### 🏗️ Features

- feat: get tone offset info

### 🪚 Maintainence

- chore: cleanup debug derivation

## Release Notes for v0.9.1

### 🪚 Maintainence

- chore: bump versions
- chore: cleanup types, remove tone info
- chore: prep for note spacing

## Release Notes for v0.9.0

### 🏗️ Features

- feat: render rests, dotted rests
- feat: basic draw rests
- feat: generate tone info with tone offsets

### 🐞 Fixes

- fix: adjusted spacing
- fix: use default clef for simplicity
- fix: slightly bigger default spacing

### 🪚 Maintainence

- chore: cleanup lint errors
- chore: bump versions
- chore: cleanup unused imports

## Release Notes for v0.8.0

### 🏗️ Features

- feat: cleanup

### 🪚 Maintainence

- chore: bump versions
- chore: bump versions

## Release Notes for v0.7.0

### 🏗️ Features

- feat: split notation in comontime

### 🪚 Maintainence

- chore: cleanup unused args

## Release Notes for v0.6.0

### 🏗️ Features

- feat(SC-83): delete tones

### 🐞 Fixes

- fix: clear tim signature popover
- fix: only fill remaining bars if the overflow is greater than 0

### 🪚 Maintainence

- chore: bump versions

## Release Notes for v0.5.0

### 🏗️ Features

- feat: render tone debugging

## Release Notes for v0.4.0

### 🏗️ Features

- feat(SC-94): when creating time signature more subsequent time signature to avoid non-full bars

### 🐞 Fixes

- fix: unused import

## Release Notes for v0.3.0

### 🏗️ Features

- feat(SC-92): create time signature basic implimentation
- feat(SC-91): draw time sigmatures

### 🪚 Maintainence

- chore: cleanup releases.md
