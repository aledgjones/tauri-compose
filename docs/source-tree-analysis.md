# Solo Compose — Source Tree Analysis

```
compose/                              # Project root
├── index.html                        # Main window entry point
├── debugger.html                     # Debug console window
├── instrument-picker.html            # Instrument selection dialog
├── layout-settings.html              # Layout configuration dialog
├── player-settings.html              # Player/mixer settings dialog
├── plugin-settings.html              # Plugin configuration dialog
├── preferences.html                  # Application preferences dialog
├── project-settings.html             # Project metadata dialog
├── package.json                      # Node dependencies, scripts (v1.23.1)
├── tsconfig.json                     # TypeScript strict config (ES2020, react-jsx)
├── vite.config.ts                    # Multi-page Vite config (8 entry points)
├── postcss.config.js                 # Tailwind CSS 4 integration
├── eslint.config.js                  # ESLint flat config
├── styles.css                        # Global Tailwind + transparencies
├── releases.md                       # Version changelog
│
├── src/                              # ── FRONTEND (React/TypeScript) ──
│   ├── vite-env.d.ts                 # Vite type declarations
│   │
│   ├── services/                     # Business logic & state management
│   │   ├── actions.ts                # All FE→BE mutations via invoke() (888 lines)
│   │   ├── types.ts                  # All shared TypeScript types (450 lines)
│   │   ├── utils.ts                  # Helper functions (name formatting, etc.)
│   │   ├── hit-test.ts               # Frontend hit-test coordination
│   │   ├── pitch-utils.ts            # Pitch/note name utilities
│   │   ├── popovers.ts              # Popover positioning logic
│   │   ├── sequencer-utils.ts        # Piano roll / sequencer helpers
│   │   └── state/                    # React context providers
│   │       ├── context.tsx           # 4 contexts (App, Audio, Sampler, Transport) + initial values
│   │       ├── provider.tsx          # AppStateProvider component + event listeners + RAF polling
│   │       └── index.ts              # Barrel export
│   │
│   ├── windows/                      # Per-window application code
│   │   ├── main/                     # ★ Primary composition window
│   │   │   ├── boot.tsx              # React mount point (AppStateProvider + TokenizationProvider)
│   │   │   ├── index.tsx             # Main layout: tabs, toolbar, transport, content area
│   │   │   ├── config.ts             # Main window configuration
│   │   │   ├── screens/
│   │   │   │   ├── setup/            # Setup tab: player/flow/layout management
│   │   │   │   │   ├── index.tsx
│   │   │   │   │   └── components/   # Panel components (player, flow, layout panels)
│   │   │   │   ├── write/            # Write tab: notation editor
│   │   │   │   │   ├── index.tsx
│   │   │   │   │   └── components/   # Bottom panel, left panel
│   │   │   │   └── sequence/         # Sequence tab: piano roll sequencer
│   │   │   │       ├── index.tsx
│   │   │   │       └── components/   # Piano roll, instrument overview, bottom panel
│   │   │   ├── components/           # Main window shared components
│   │   │   │   ├── renderer.tsx      # Native score surface overlay (transparent hit-test div)
│   │   │   │   ├── transport.tsx     # Playback transport controls
│   │   │   │   ├── tools.tsx         # Tool selection bar
│   │   │   │   ├── sequencer.tsx     # Piano roll component
│   │   │   │   ├── sequencer-track.tsx
│   │   │   │   ├── sequencer-track-tone.tsx
│   │   │   │   ├── mixer.tsx         # Mixer panel
│   │   │   │   ├── mixer-channel.tsx
│   │   │   │   ├── fader.tsx         # Volume fader control
│   │   │   │   ├── keyboard.tsx      # Piano keyboard display
│   │   │   │   ├── playback-line.tsx # Playback position indicator
│   │   │   │   ├── tick-bar.tsx      # Tick/time ruler
│   │   │   │   ├── plugins.tsx       # Plugin management panel
│   │   │   │   ├── zoom-widget.tsx   # Zoom controls
│   │   │   │   └── selected-tone-info-display.tsx
│   │   │   ├── hooks/                # Main window hooks
│   │   │   │   ├── use-bottom-panel-state.ts
│   │   │   │   ├── use-keyboard-shortcuts.ts
│   │   │   │   ├── use-selected-tone-info.ts
│   │   │   │   ├── use-slots.ts
│   │   │   │   └── use-ticks.ts
│   │   │   └── utils/
│   │   │       └── repeat-array.ts
│   │   ├── debugger/                 # Debug console window
│   │   ├── instrument-picker/        # Instrument picker dialog
│   │   ├── layout-settings/          # Layout settings dialog
│   │   ├── player-settings/          # Player settings dialog
│   │   ├── plugin-settings/          # Plugin settings dialog
│   │   ├── preferences/              # Preferences dialog
│   │   └── project-settings/         # Project settings dialog
│   │
│   ├── components/                   # Shared app-level components
│   │   ├── app-bar.tsx               # Title bar component
│   │   ├── drag-scroll.tsx           # Drag-to-scroll container
│   │   ├── not-found.tsx             # 404 / route-not-found
│   │   ├── select-by-image.tsx       # Image-based selection widget
│   │   └── snap-picker.tsx           # Snap duration picker
│   │
│   ├── ui/                           # Reusable UI primitives
│   │   ├── buttons/
│   │   │   ├── button.tsx            # Standard button
│   │   │   ├── icon-button.tsx       # Icon-only button
│   │   │   ├── bravura-icon-button.tsx  # Music notation icon button
│   │   │   ├── shared.ts            # Shared button utilities
│   │   │   └── smufl.ts             # SMuFL glyph mappings
│   │   ├── input/
│   │   │   ├── index.tsx            # Base input wrapper
│   │   │   ├── number-input.tsx     # Number input with validation
│   │   │   └── text-input.tsx       # Text input
│   │   ├── text/
│   │   │   ├── context.ts           # Text tokenization context
│   │   │   ├── index.tsx            # Text display (with music glyph support)
│   │   │   └── provider.tsx         # Tokenization provider
│   │   ├── hooks/
│   │   │   └── use-drag-handler/    # Drag interaction utilities
│   │   ├── checkbox.tsx
│   │   ├── icon.tsx                 # MDI icon wrapper
│   │   ├── menu.tsx                 # Context menu
│   │   ├── popover.tsx              # Popover container
│   │   ├── select.tsx               # Select dropdown
│   │   ├── switch.tsx               # Toggle switch
│   │   ├── tabs.tsx                 # Tab bar
│   │   └── textarea.tsx             # Multi-line text input
│   │
│   ├── instruments/                  # Frontend instrument metadata
│   │   └── dist/                     # Compiled instrument data
│   │
│   ├── utils/
│   │   └── mm.ts                    # Millimeter conversion utilities
│   │
│   └── assets/
│       └── fonts/                    # Bravura music notation font
│
├── src-tauri/                        # ── BACKEND (Rust/Tauri) ──
│   ├── Cargo.toml                    # Rust dependencies (v1.23.1)
│   ├── tauri.conf.json               # Tauri config (window, bundle, security)
│   ├── build.rs                      # Tauri build script
│   ├── src/
│   │   ├── main.rs                   # Entry point (calls compose_lib::run())
│   │   ├── lib.rs                    # App setup: managed state, commands, menus (552 lines)
│   │   ├── files.rs                  # File I/O: save/load .soloc, recent files, migrations (405 lines)
│   │   ├── preferences.rs            # User preferences (audition toggle)
│   │   ├── utils.rs                  # UniqueId type, timestamp helper
│   │   │
│   │   ├── score/                    # ★ Score data model
│   │   │   ├── mod.rs                # Score struct (top-level container)
│   │   │   ├── config.rs             # AutoCountStyles configuration
│   │   │   ├── meta.rs               # Project metadata (title, composer, etc.)
│   │   │   ├── players.rs            # Player management (Solo/Section)
│   │   │   ├── flows.rs              # Flow management (movements)
│   │   │   ├── tracks.rs             # Track + Entry storage
│   │   │   ├── staves.rs             # Stave definitions
│   │   │   ├── layouts.rs            # Score layouts (FullScore/Part)
│   │   │   ├── engraves.rs           # Engraving settings
│   │   │   ├── entries/              # Entry type definitions
│   │   │   │   ├── mod.rs            # Entry enum + Entries struct
│   │   │   │   ├── tone.rs           # Tone entry + CRUD commands
│   │   │   │   ├── barline.rs        # Barline entry + draw types
│   │   │   │   ├── clef.rs           # Clef entry + glyphs
│   │   │   │   ├── key_signature.rs  # Key signature + sharps/flats tables
│   │   │   │   └── time_signature.rs # Time signature + groupings
│   │   │   ├── instruments/
│   │   │   │   ├── mod.rs            # Instrument struct + management
│   │   │   │   └── defs.rs           # Hardcoded instrument definitions (744 lines)
│   │   │   └── components/           # Shared value types
│   │   │       ├── mod.rs
│   │   │       ├── duration.rs       # Tick/Ticks types, NoteDuration enum
│   │   │       ├── pitch.rs          # Pitch + Accidental + accidental maps (719 lines)
│   │   │       ├── velocity.rs       # Velocity (u8 wrapper)
│   │   │       ├── articulation.rs   # Articulation enum + exclusivity groups
│   │   │       ├── measurements.rs   # BoundingBox, Padding, Point types
│   │   │       └── text.rs           # Font, Justify, Align, FontFamily
│   │   │
│   │   ├── instructions/             # ★ Score render pipeline
│   │   │   ├── mod.rs                # generate_render_instructions() orchestrator (667 lines)
│   │   │   ├── parse/                # Parse phase (19 modules)
│   │   │   │   ├── mod.rs
│   │   │   │   ├── get_measures.rs
│   │   │   │   ├── get_written_durations.rs
│   │   │   │   ├── get_tone_offsets.rs
│   │   │   │   ├── get_tone_offset_info.rs
│   │   │   │   ├── get_horizontal_spacing.rs
│   │   │   │   ├── get_vertical_spacing.rs
│   │   │   │   ├── get_stem_directions.rs
│   │   │   │   ├── get_stem_lengths.rs
│   │   │   │   ├── get_beams.rs
│   │   │   │   ├── get_accidentals.rs
│   │   │   │   ├── get_articulations.rs
│   │   │   │   ├── get_dots.rs
│   │   │   │   ├── get_shunts.rs
│   │   │   │   ├── get_barlines.rs
│   │   │   │   ├── get_key_signatures.rs
│   │   │   │   ├── get_instruments.rs
│   │   │   │   ├── get_vertical_spans.rs
│   │   │   │   ├── get_max_brackets_width.rs
│   │   │   │   └── get_max_instrument_name_width.rs
│   │   │   └── draw/                 # Draw phase (20 modules)
│   │   │       ├── mod.rs
│   │   │       ├── draw_notehead.rs
│   │   │       ├── draw_stem.rs
│   │   │       ├── draw_beam.rs
│   │   │       ├── draw_flag.rs
│   │   │       ├── draw_rest.rs
│   │   │       ├── draw_dot.rs
│   │   │       ├── draw_clef.rs
│   │   │       ├── draw_key_signature.rs
│   │   │       ├── draw_time_signature.rs
│   │   │       ├── draw_barline.rs
│   │   │       ├── draw_accidentals.rs
│   │   │       ├── draw_articulations.rs
│   │   │       ├── draw_ties.rs
│   │   │       ├── draw_ledger_lines.rs
│   │   │       ├── draw_staves.rs
│   │   │       ├── draw_names.rs
│   │   │       ├── draw_braces.rs
│   │   │       ├── draw_brackets.rs
│   │   │       ├── draw_sub_brackets.rs
│   │   │       └── draw_systemic_barline.rs
│   │   │
│   │   ├── renderer/                 # ★ Native rendering engine
│   │   │   ├── mod.rs                # RendererContext, DrawInstruction enum, HitTarget (308 lines)
│   │   │   ├── backend.rs            # RenderBackend trait
│   │   │   ├── backends/
│   │   │   │   ├── mod.rs
│   │   │   │   └── metal.rs          # MetalRenderer (Skia + CAMetalLayer)
│   │   │   ├── converter.rs          # Space↔Px↔Mm converter
│   │   │   ├── fonts.rs              # Font loading (Bravura, LibreBaskerville)
│   │   │   ├── hit_test.rs           # Click-to-entry resolution
│   │   │   └── text_measurer.rs      # Text metrics for layout
│   │   │
│   │   ├── playback/                 # ★ Audio engine
│   │   │   ├── mod.rs
│   │   │   ├── audio.rs              # cpal device management (280 lines)
│   │   │   ├── transport.rs          # Transport thread + lock-free state (551 lines)
│   │   │   ├── sampler.rs            # Plugin scanning + instrument caching (507 lines)
│   │   │   ├── mixer.rs              # 64-slot lock-free voice mixer (163 lines)
│   │   │   ├── streamer.rs           # Disk-streaming FLAC decode + resample (448 lines)
│   │   │   └── scvi.rs               # SCVI binary container reader (217 lines)
│   │   │
│   │   ├── ui/                       # UI state management (Rust side)
│   │   │   ├── mod.rs                # Ui struct, Tab enum, tab/layout/flow commands
│   │   │   ├── renderer.rs           # Renderer UI state (zoom, pan, bounds, debug)
│   │   │   ├── sequencer.rs          # Sequencer UI state (tool, snap, selection)
│   │   │   └── screens/
│   │   │       └── sequence.rs       # Sequence screen state
│   │   │
│   │   ├── migrations/               # File format migrations
│   │   │   └── mod.rs                # Migration registry
│   │   │
│   │   └── bin/                      # CLI tools (compiled separately)
│   │       ├── scvi_compress.rs      # Compress instruments → SCVI format
│   │       ├── scvi_inspect.rs       # Inspect SCVI container contents
│   │       └── generate_manifest.rs  # Generate instrument manifests
│   │
│   ├── capabilities/                 # Tauri security capabilities
│   ├── gen/                          # Tauri generated code
│   ├── icons/                        # App icons (icns, ico, png)
│   └── target/                       # Rust build output
│
├── instruments/                      # ★ Instrument sample libraries
│   ├── bass drum/                    # manifest.json + samples/
│   ├── bongo/
│   ├── drum kit/
│   ├── flute 1/
│   ├── gong/
│   ├── harpsichord/
│   ├── piano/                        # manifest.json (1183 lines) + Steinway samples
│   ├── tubular bells/
│   ├── viola/
│   ├── violin 1/
│   ├── violin 2/
│   └── violoncello/
│
├── legacy-instruments/               # Older instrument format
│   ├── bassoon/                      # manifest.json + sustain/
│   └── clarinet/
│
├── scripts/                          # Build tools (compiled Rust binaries)
│   ├── generate-manifest             # Instrument manifest generator
│   ├── release                       # Version bump + release tool
│   ├── scvi-compress                 # SCVI compression tool
│   └── scvi-inspect                  # SCVI inspection tool
│
└── docs/                             # Project documentation
    └── project-context.md            # AI agent rules (52 rules)
```
