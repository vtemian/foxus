import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// Mock Tauri invoke API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
