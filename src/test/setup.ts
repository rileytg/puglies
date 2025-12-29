// AIDEV-NOTE: Test setup - configures jsdom, jest-dom matchers, and global mocks
import "@testing-library/jest-dom/vitest";
import { afterEach, beforeEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import { _resetForTesting } from "@/lib/backend";

// Cleanup after each test
afterEach(() => {
  cleanup();
});

// Reset all mocks before each test
beforeEach(() => {
  vi.clearAllMocks();
});

// Reset backend singletons before each test
beforeEach(() => {
  _resetForTesting();
});

// Mock window.__TAURI_INTERNALS__ for tests
// Tests can override this per-test as needed
beforeEach(() => {
  // Default: not in Tauri environment
  delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;
});
