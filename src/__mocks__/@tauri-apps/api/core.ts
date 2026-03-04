import { vi } from "vitest";

/**
 * Manual mock for @tauri-apps/api/core.
 * Intercepts all invoke() calls — no real Tauri IPC is used in tests.
 *
 * Override per-test with:
 *   invoke.mockResolvedValueOnce(data)
 *   invoke.mockImplementation((cmd) => { ... })
 */
export const invoke = vi.fn().mockResolvedValue(undefined);
