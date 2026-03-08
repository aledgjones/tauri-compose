# Solo Compose — UI Component Inventory

## Overview

The UI is split into three tiers:

1. **`src/ui/`** — Reusable design system primitives (buttons, inputs, selects, tabs)
2. **`src/components/`** — Shared app-level components (app bar, drag scroll, snap picker)
3. **`src/windows/*/components/`** — Window-specific components

All components are functional `FC<Props>`. Styling uses Tailwind CSS 4 utility classes + `classnames`. Dark theme: neutral-900 base, sky-500 accents, transparent backgrounds. All `cursor: default` globally for native feel.

---

## Design System Primitives (`src/ui/`)

### Buttons

| Component           | File                              | Description                               |
| ------------------- | --------------------------------- | ----------------------------------------- |
| `Button`            | `buttons/button.tsx`              | Standard button with variants             |
| `IconButton`        | `buttons/icon-button.tsx`         | Icon-only button (MDI icons)              |
| `BravuraIconButton` | `buttons/bravura-icon-button.tsx` | Music notation icon button (SMuFL glyphs) |

Supporting: `shared.ts` (shared button styles), `smufl.ts` (SMuFL glyph code mappings)

### Inputs

| Component     | File                     | Description                   |
| ------------- | ------------------------ | ----------------------------- |
| `Input`       | `input/index.tsx`        | Base input wrapper            |
| `NumberInput` | `input/number-input.tsx` | Numeric input with validation |
| `TextInput`   | `input/text-input.tsx`   | Text input                    |
| `Textarea`    | `textarea.tsx`           | Multi-line text area          |

### Selection & Navigation

| Component  | File           | Description        |
| ---------- | -------------- | ------------------ |
| `Select`   | `select.tsx`   | Dropdown select    |
| `Tabs`     | `tabs.tsx`     | Tab bar navigation |
| `Menu`     | `menu.tsx`     | Context menu       |
| `Popover`  | `popover.tsx`  | Popover container  |
| `Checkbox` | `checkbox.tsx` | Checkbox input     |
| `Switch`   | `switch.tsx`   | Toggle switch      |

### Display

| Component              | File                | Description                                          |
| ---------------------- | ------------------- | ---------------------------------------------------- |
| `Icon`                 | `icon.tsx`          | MDI icon wrapper (@mdi/react)                        |
| `Text`                 | `text/index.tsx`    | Text display with music glyph tokenization           |
| `TokenizationProvider` | `text/provider.tsx` | Provides text tokenization context for music symbols |

### Hooks

| Hook             | Location                  | Description                     |
| ---------------- | ------------------------- | ------------------------------- |
| `useDragHandler` | `hooks/use-drag-handler/` | Reusable drag interaction logic |

---

## Shared App Components (`src/components/`)

| Component       | File                  | Description                                              |
| --------------- | --------------------- | -------------------------------------------------------- |
| `AppBar`        | `app-bar.tsx`         | macOS-style title bar with traffic lights offset         |
| `DragScroll`    | `drag-scroll.tsx`     | Click-and-drag scrollable container                      |
| `NotFound`      | `not-found.tsx`       | Fallback for unknown routes/tabs                         |
| `SelectByImage` | `select-by-image.tsx` | Image-based selection widget (e.g., barline type picker) |
| `SnapPicker`    | `snap-picker.tsx`     | Note duration snap selector                              |

---

## Main Window Components (`src/windows/main/components/`)

### Core

| Component    | File              | Description                                                                                                               |
| ------------ | ----------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `Renderer`   | `renderer.tsx`    | Transparent overlay div for native score surface — handles resize, pan, zoom, click events, delegates hit-testing to Rust |
| `Transport`  | `transport.tsx`   | Playback controls (play, pause, stop, seek, tempo)                                                                        |
| `Tools`      | `tools.tsx`       | Tool selection bar (Select, Write, Slice, Erase)                                                                          |
| `ZoomWidget` | `zoom-widget.tsx` | Score zoom controls                                                                                                       |

### Sequencer / Piano Roll

| Component                 | File                             | Description                                      |
| ------------------------- | -------------------------------- | ------------------------------------------------ |
| `Sequencer`               | `sequencer.tsx`                  | Piano roll editor container                      |
| `SequencerTrack`          | `sequencer-track.tsx`            | Single track row in piano roll                   |
| `SequencerTrackTone`      | `sequencer-track-tone.tsx`       | Individual tone block in piano roll              |
| `Keyboard`                | `keyboard.tsx`                   | Piano keyboard display (left side of piano roll) |
| `TickBar`                 | `tick-bar.tsx`                   | Time ruler / tick position indicator             |
| `PlaybackLine`            | `playback-line.tsx`              | Vertical playback position line                  |
| `SelectedToneInfoDisplay` | `selected-tone-info-display.tsx` | Shows details of selected tone                   |

### Mixer

| Component      | File                | Description                                     |
| -------------- | ------------------- | ----------------------------------------------- |
| `Mixer`        | `mixer.tsx`         | Mixer panel container                           |
| `MixerChannel` | `mixer-channel.tsx` | Single channel strip with fader, pan, mute/solo |
| `Fader`        | `fader.tsx`         | Vertical volume fader control                   |

### Other

| Component | File          | Description             |
| --------- | ------------- | ----------------------- |
| `Plugins` | `plugins.tsx` | Plugin management panel |

---

## Main Window Screens

### Setup Screen (`screens/setup/`)

Manages players, instruments, flows, and layouts.

| Component                       | File                                               | Description                                  |
| ------------------------------- | -------------------------------------------------- | -------------------------------------------- |
| `Setup`                         | `index.tsx`                                        | Three-panel layout (players, flows, layouts) |
| `PlayerPanel`                   | `components/player-panel.tsx`                      | Player list with instruments                 |
| `PlayerPanelItem`               | `components/player-panel-item.tsx`                 | Single player row                            |
| `PlayerPanelInstrumentListItem` | `components/player-panel-instrument-list-item.tsx` | Instrument within a player                   |
| `FlowPanel`                     | `components/flow-panel.tsx`                        | Flow (movement) list                         |
| `FlowPanelItem`                 | `components/flow-panel-item.tsx`                   | Single flow row                              |
| `LayoutPanel`                   | `components/layout-panel.tsx`                      | Layout list                                  |
| `LayoutPanelItem`               | `components/layout-panel-item.tsx`                 | Single layout row                            |
| `Panel`                         | `components/panel.tsx`                             | Shared panel container                       |

### Write Screen (`screens/write/`)

The notation editor view.

| Component                | File                                      | Description                                                |
| ------------------------ | ----------------------------------------- | ---------------------------------------------------------- |
| `Write`                  | `index.tsx`                               | Score view with collapsible bottom panel                   |
| `BottomPanel`            | `components/bottom-panel.tsx`             | Properties/Sequencer/Mixer/Instruments tabs                |
| `PropertiesPanel`        | `components/properties-panel.tsx`         | Selection-aware shell for write-screen properties content  |
| `TempoPropertiesSection` | `components/tempo-properties-section.tsx` | Immediate tempo editing controls inside the Properties tab |
| `LeftPanel`              | `components/left-panel.tsx`               | Tool properties panel                                      |

### Sequence Screen (`screens/sequence/`)

Piano roll / sequencer view.

| Component               | File                                     | Description                                     |
| ----------------------- | ---------------------------------------- | ----------------------------------------------- |
| `Sequence`              | `index.tsx`                              | Full-height piano roll with instrument overview |
| `BottomPanel`           | `components/bottom-panel.tsx`            | Properties/Sequencer/Mixer/Instruments tabs     |
| `InstrumentOverview`    | `components/instrument-overview.tsx`     | Instrument list with miniature piano rolls      |
| `InstrumentOverviewRow` | `components/instrument-overview-row.tsx` | Single instrument row                           |
| `MiniaturePianoRoll`    | `components/miniature-piano-roll.tsx`    | Small preview of instrument track               |

---

## Main Window Hooks

| Hook                   | File                        | Description                                                   |
| ---------------------- | --------------------------- | ------------------------------------------------------------- |
| `useBottomPanelState`  | `use-bottom-panel-state.ts` | Manages bottom panel tab and minimized state                  |
| `useKeyboardShortcuts` | `use-keyboard-shortcuts.ts` | Global keyboard shortcut handler                              |
| `useSelectedTempo`     | `use-selected-tempo.ts`     | Resolves the selected tempo entry from flow master-track data |
| `useSelectedToneInfo`  | `use-selected-tone-info.ts` | Resolves selected tone details                                |
| `useSlots`             | `use-slots.ts`              | Manages slot allocation                                       |
| `useTicks`             | `use-ticks.ts`              | Tick position calculation                                     |
