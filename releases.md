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
