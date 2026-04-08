# Solo Compose — Documentation Index

> **Music composition simplified.** Native desktop app for creating, sequencing, and engraving musical scores.

**Version:** 1.23.1 | **Author:** Aled Jones | **Stack:** Tauri 2 + React 19 + Rust + Skia/Metal

---

## Quick Reference

- **Type:** Desktop (Tauri 2, monolith)
- **Primary Languages:** TypeScript (frontend), Rust (backend)
- **Architecture:** Rust-owned state → React display layer, native score rendering via Skia/Metal
- **Entry Point (Frontend):** `src/windows/main/boot.tsx` (+ 7 secondary window entry points)
- **Entry Point (Backend):** `src-tauri/src/main.rs` → `lib.rs`

---

## Generated Documentation

- [Project Overview](./project-overview.md) — Executive summary, features, tech stack
- [Architecture](./architecture.md) — System architecture, state management, render pipeline, audio pipeline
- [Data Models](./data-models.md) — Score data model, Entry system, dual-index storage, relationships
- [Source Tree Analysis](./source-tree-analysis.md) — Complete annotated directory tree
- [Component Inventory](./component-inventory.md) — UI primitives, app components, screen components
- [Development Guide](./development-guide.md) — Setup, commands, versioning, build config
- [Render Performance Baselines](./performance-baselines.md) — Historical render benchmark snapshots and comparison guidance
- [MVP Playback Support Baseline](./playback-support-baseline.md) — Canonical bounded plugin, scenario, and evidence baseline for Epic 7 release decisions

---

## Existing Documentation

- [README.md](../README.md) — Project introduction, setup guide, design principles
- [Project Context](./project-context.md) — 52 implementation rules for AI agents (serialization, architecture, render pipeline gotchas)
- [Releases](../releases.md) — Version changelog

---

## Getting Started

```bash
# Install dependencies
npm install

# Start development mode
npm run start:dev

# Validate before committing
npm run lint && npm run check-types && cd src-tauri && cargo clippy
```

For detailed setup instructions, see the [Development Guide](./development-guide.md).

---

## Key Architecture Decisions

1. **Rust owns all state** — the frontend is a display layer only. All mutations go through `invoke()` in `actions.ts`.
2. **Score rendered natively** — Skia/Metal paints the score page. React handles everything else (panels, buttons, toolbars).
3. **Transport uses RAF polling** — no events emitted during playback to keep the transport thread silent.
4. **FxHashMap everywhere** — never `std::collections::HashMap`. Measured performance benefit for the project's key types.
5. **Multi-window via Tauri events** — `app.emit()` pushes state to all 8 windows simultaneously.

---

_Generated: 2026-03-04 | Scan level: deep | Workflow: document-project v1.2.0_
