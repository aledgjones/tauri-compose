# Render Performance Report

This file keeps maintainer-facing render benchmark history in a report format so individual runs stay visible while long-term trends remain easy to scan.

## Executive Summary

- Latest recorded branch snapshot: 2026-03-19 on commit `fa6f67ff`
- Current representative single-flow median: `949us`
- Current baseline warning threshold: `2000us`
- Current status: within threshold
- Current branch trends faster than the reproduced pre-tuplet baseline across all recorded scenarios

## Trend Overview

| Snapshot                       | Measured On | Commit     | Kind                    | Samples | Baseline Median | Shared Two-Flow Median | Local Edit Median | Delta vs Previous Snapshot                                       |
| ------------------------------ | ----------- | ---------- | ----------------------- | ------: | --------------: | ---------------------: | ----------------: | ---------------------------------------------------------------- |
| Pre-tuplet benchmark baseline  | 2026-03-19  | `063efd68` | Historical reproduction |      15 |        `1097us` |               `2051us` |          `1052us` | Starting point                                                   |
| Current `feat/triplets` branch | 2026-03-19  | `fa6f67ff` | Live branch measurement |      15 |         `949us` |               `1711us` |           `916us` | Baseline `-148us`, shared two-flow `-340us`, local edit `-136us` |

## Methodology

- Use `npm run perf:render -- --samples 15 --json` when recording a durable snapshot.
- Treat the reported total median as the primary comparison value for each scenario.
- Record whether the run is a live branch measurement or a historical reproduction.
- Keep environment notes explicit. Reproduced historical runs are useful, but they are not equivalent to an original archived measurement from that date.
- Append new run records rather than replacing older ones so the trend table remains auditable.

## Scenario Definitions

| Scenario          | Purpose                                                                    | Expected Scope                                          |
| ----------------- | -------------------------------------------------------------------------- | ------------------------------------------------------- |
| `baseline`        | Representative single-flow render pass for the current release-like target | 1 visible flow, 1 recalculated flow, 1 composed flow    |
| `shared-two-flow` | Shared-surface multi-flow render pass                                      | 2 visible flows, 2 recalculated flows, 2 composed flows |
| `local-edit`      | Local edit pass with reuse available                                       | 3 visible flows, 1 recalculated flow, 3 composed flows  |

## Latest Snapshot

### 2026-03-19 Live Branch Measurement (`fa6f67ff`)

Command:

```bash
npm run perf:render -- --samples 15 --json
```

Summary:

- Branch state after current tuplet render work
- Representative baseline remains below the `2000us` warning threshold
- All recorded medians are lower than the reproduced pre-tuplet snapshot on the same machine and day

| Scenario        | Request   | Flows | Recalculated / Composed | Total Min | Total Median | Total Max | Segment Median | Compose Median |
| --------------- | --------- | ----: | ----------------------: | --------: | -----------: | --------: | -------------: | -------------: |
| baseline        | `full`    |     1 |                   1 / 1 |   `754us` |      `949us` |  `1196us` |        `937us` |         `12us` |
| shared-two-flow | `full`    |     2 |                   2 / 2 |  `1547us` |     `1711us` |  `2311us` |       `1687us` |         `23us` |
| local-edit      | `flow 20` |     3 |                   1 / 3 |   `795us` |      `916us` |  `1006us` |        `876us` |         `34us` |

Baseline warning status:

- Threshold: `2000us` median for the representative single-flow pass
- Result: within threshold (`949us`)

## Historical Run Log

### 2026-03-19 Historical Reproduction (`063efd68`)

Command:

```bash
npm run perf:render -- --samples 15 --json
```

Execution notes:

- Run from an isolated git worktree at commit `063efd68`
- Benchmark infrastructure commit before tuplet work began
- Reproduced historical measurement, not an original archived artifact from that earlier branch state

| Scenario        | Request   | Flows | Recalculated / Composed | Total Min | Total Median | Total Max | Segment Median | Compose Median |
| --------------- | --------- | ----: | ----------------------: | --------: | -----------: | --------: | -------------: | -------------: |
| baseline        | `full`    |     1 |                   1 / 1 |   `903us` |     `1097us` |  `1243us` |       `1083us` |         `11us` |
| shared-two-flow | `full`    |     2 |                   2 / 2 |  `1829us` |     `2051us` |  `2333us` |       `2026us` |         `24us` |
| local-edit      | `flow 20` |     3 |                   1 / 3 |   `961us` |     `1052us` |  `1221us` |       `1013us` |         `39us` |

Baseline warning status:

- Threshold: `2000us` median for the representative single-flow pass
- Result: within threshold (`1097us`)

## Interpretation Notes

- The current branch trends faster than the reproduced pre-tuplet baseline across baseline, shared-two-flow, and local-edit scenarios.
- These comparisons are useful because both snapshots were measured on the same machine, on the same day, with the same command and sample count.
- The historical row should still be read as a reproduction, not as an original point-in-time benchmark artifact.
