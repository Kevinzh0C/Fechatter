import { vi } from 'vitest';

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
global.localStorage = localStorageMock;

// Mock crypto only if not already defined
if (!global.crypto) {
  global.crypto = {
    randomUUID: () => '123e4567-e89b-12d3-a456-426614174000'
  };
}

// Reset all mocks before each test
beforeEach(() => {
  vi.clearAllMocks();
  localStorage.getItem.mockReset();
  localStorage.setItem.mockReset();
  localStorage.removeItem.mockReset();
});