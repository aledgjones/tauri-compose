# Solo Compose — Project Overview

> **Music composition simplified.** A native desktop application for creating, sequencing, and engraving musical scores.

**Version:** 1.23.1
**Author:** Aled Jones
**Identifier:** `com.soloapps.compose`

---

## Executive Summary

Solo Compose is a Tauri-based desktop application that combines a React/TypeScript frontend with a Rust backend to deliver a professional music composition workflow. The app features five operational modes — Setup, Write, Engrave, Sequence, and Print — with hardware-accelerated score rendering via Skia/Metal. Playback direction is VST-first, while any remaining `.scvi` support is bounded to temporary legacy internal sampler seams rather than the supported product path. The explicit end state is full SCVI removal once the VST replacement path is complete.

## Architecture Type

- **Repository:** Monolith
- **Project Type:** Desktop (Tauri 2)
- **Pattern:** Native-rendered score surface + React UI shell
- **State Model:** Rust-owned state pushed to all frontend windows via Tauri events

## Tech Stack Summary

| Layer         | Technology                | Version    |
| ------------- | ------------------------- | ---------- |
| Frontend      | React + TypeScript        | 19.2 / 5.9 |
| Bundler       | Vite                      | 7.3        |
| Styling       | Tailwind CSS              | 4.2        |
| Native Shell  | Tauri                     | 2.10       |
| Backend       | Rust (2021 edition)       | stable     |
| Rendering     | skia-safe (Metal)         | 0.93       |
| Audio I/O     | cpal                      | 0.17       |
| Audio Decode  | symphonia (FLAC) + rubato | 0.5 / 1.0  |
| Storage       | sled (embedded KV)        | 0.34       |
| Serialization | serde + serde_repr        | 1.0        |

## Key Feature Areas

1. **Notation Editor** — Professional score writing with multiple clef types, key/time signatures, barlines, articulations
2. **Score Engraving** — Hardware-accelerated rendering via 19-module parse pipeline → 20-module draw pipeline → Metal backend
3. **Audio Sequencing** — Piano roll editor with multi-track instrument sequencing, expression control
4. **Playback Direction** — VST-first playback architecture is the intended product direction; the current product path remains foundation-building while legacy `.scvi` container support is retained only for bounded temporary internal sampler seams, and current VST work must replace rather than extend those seams
5. **Multi-Window UI** — 10 HTML entry points (main, debugger, instrument-picker, layout-settings, player-settings, plugin-host-poc, plugin-manager, plugin-settings, preferences, project-settings)
6. **Instrument Library** — 12+ instrument patches (orchestral strings, keyboards, percussion, woodwinds) with multi-velocity sampling

## File Format

Projects are saved as `.soloc` files — a custom binary container with `SOLOC` magic, a container header, a chunk directory, and a required gzip-compressed `PROJECT_JSON` chunk containing `{ ui, score, next_id }` as minified JSON. The container header `app_version` is the authoritative migration key, while `score.meta.app_version` remains in the JSON payload for inspection and must match the header value. A migration system (`migrations/`) continues to transform JSON payloads across compatible app versions.
