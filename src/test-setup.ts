/// <reference types="vitest/globals" />
import "@testing-library/jest-dom/vitest";

// Suppress console.log and console.info during tests to keep output clean.
// Run `npm run test:js:logs` to disable this and see all output.
if (!process.env.VITEST_LOGS) {
  vi.spyOn(console, "log").mockImplementation(() => {});
  vi.spyOn(console, "info").mockImplementation(() => {});
}
