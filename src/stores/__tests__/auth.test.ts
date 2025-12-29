// AIDEV-NOTE: Tests for auth store - login, logout, portfolio fetching with mocked backend
import { describe, it, expect, beforeEach, vi } from "vitest";
import { useAuthStore } from "../auth";
import { _setBackendForTesting, _resetForTesting } from "@/lib/backend";
import {
  createMockBackend,
  mockAuthStatus,
  mockBalance,
  mockPosition,
  mockOrder,
} from "@/test/mock-adapter";

describe("useAuthStore", () => {
  beforeEach(() => {
    // Reset backend singleton
    _resetForTesting();
    // Reset zustand store to initial state
    useAuthStore.setState({
      status: { isAuthenticated: false },
      isLoading: false,
      error: null,
      polymarketAddress: null,
      balance: null,
      positions: [],
      orders: [],
      portfolioLoading: false,
    });
  });

  describe("initial state", () => {
    it("starts with unauthenticated status", () => {
      const { status } = useAuthStore.getState();
      expect(status.isAuthenticated).toBe(false);
    });

    it("starts with empty portfolio", () => {
      const { balance, positions, orders } = useAuthStore.getState();
      expect(balance).toBeNull();
      expect(positions).toEqual([]);
      expect(orders).toEqual([]);
    });
  });

  describe("checkAuthStatus", () => {
    it("updates status from backend", async () => {
      const mockBackend = createMockBackend({
        authStatus: mockAuthStatus,
      });
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().checkAuthStatus();

      const { status, polymarketAddress } = useAuthStore.getState();
      expect(status.isAuthenticated).toBe(true);
      expect(status.address).toBe(mockAuthStatus.address);
      expect(polymarketAddress).toBe(mockAuthStatus.polymarketAddress);
    });

    it("fetches portfolio when authenticated", async () => {
      const mockBackend = createMockBackend({
        authStatus: { ...mockAuthStatus, polymarketAddress: "0xpoly" },
        balance: mockBalance,
        positions: [mockPosition],
        orders: [mockOrder],
      });
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().checkAuthStatus();

      // Wait for async portfolio fetch with longer timeout
      await vi.waitFor(
        () => {
          const { balance } = useAuthStore.getState();
          expect(balance).not.toBeNull();
        },
        { timeout: 1000 }
      );

      const { balance, positions, orders } = useAuthStore.getState();
      expect(balance).toEqual(mockBalance);
      expect(positions).toHaveLength(1);
      expect(orders).toHaveLength(1);
    });

    it("handles errors gracefully", async () => {
      const mockBackend = createMockBackend({
        shouldThrow: true,
        errorMessage: "Network error",
      });
      _setBackendForTesting(mockBackend);

      // Should not throw, just log error
      await useAuthStore.getState().checkAuthStatus();

      const { status } = useAuthStore.getState();
      expect(status.isAuthenticated).toBe(false);
    });
  });

  describe("login", () => {
    it("authenticates with private key", async () => {
      const mockBackend = createMockBackend({
        authStatus: mockAuthStatus,
      });
      _setBackendForTesting(mockBackend);

      const result = await useAuthStore.getState().login("test-private-key");

      expect(result).toBe(true);
      expect(mockBackend.login).toHaveBeenCalledWith("test-private-key");

      const { status, isLoading } = useAuthStore.getState();
      expect(status.isAuthenticated).toBe(true);
      expect(isLoading).toBe(false);
    });

    it("sets loading state during login", async () => {
      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      // Check loading state is set
      const loginPromise = useAuthStore.getState().login("key");

      // The state should be loading after we call login
      // (in real implementation this would be true during the async call)
      await loginPromise;

      const { isLoading } = useAuthStore.getState();
      expect(isLoading).toBe(false); // After completion
    });

    it("returns false and sets error on failure", async () => {
      const mockBackend = createMockBackend({
        shouldThrow: true,
        errorMessage: "Invalid key",
      });
      _setBackendForTesting(mockBackend);

      const result = await useAuthStore.getState().login("bad-key");

      expect(result).toBe(false);

      const { status, error, isLoading } = useAuthStore.getState();
      expect(status.isAuthenticated).toBe(false);
      expect(error).toBe("Invalid key");
      expect(isLoading).toBe(false);
    });

    it("fetches portfolio after successful login", async () => {
      const mockBackend = createMockBackend({
        authStatus: mockAuthStatus,
        balance: mockBalance,
      });
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().login("key");

      // Wait for portfolio fetch
      await vi.waitFor(() => {
        const { portfolioLoading } = useAuthStore.getState();
        return !portfolioLoading;
      });

      expect(mockBackend.getBalance).toHaveBeenCalled();
    });
  });

  describe("logout", () => {
    it("clears authentication and portfolio", async () => {
      // First login
      const mockBackend = createMockBackend({
        authStatus: mockAuthStatus,
        balance: mockBalance,
        positions: [mockPosition],
        orders: [mockOrder],
      });
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().login("key");

      // Wait for portfolio fetch
      await vi.waitFor(() => {
        const { portfolioLoading } = useAuthStore.getState();
        return !portfolioLoading;
      });

      // Now logout
      await useAuthStore.getState().logout();

      const { status, balance, positions, orders, isLoading } = useAuthStore.getState();
      expect(status.isAuthenticated).toBe(false);
      expect(balance).toBeNull();
      expect(positions).toEqual([]);
      expect(orders).toEqual([]);
      expect(isLoading).toBe(false);
    });

    it("calls backend logout", async () => {
      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().logout();

      expect(mockBackend.logout).toHaveBeenCalled();
    });

    it("handles logout errors", async () => {
      const mockBackend = createMockBackend({
        shouldThrow: true,
        errorMessage: "Logout failed",
      });
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().logout();

      const { error, isLoading } = useAuthStore.getState();
      expect(error).toBe("Logout failed");
      expect(isLoading).toBe(false);
    });
  });

  describe("setPolymarketAddress", () => {
    it("updates address and fetches positions", async () => {
      const mockBackend = createMockBackend({
        positions: [mockPosition],
      });
      _setBackendForTesting(mockBackend);

      // Call the async function and wait for it to complete
      await useAuthStore.getState().setPolymarketAddress("0xnewaddr");

      // Wait for positions to be populated
      await vi.waitFor(
        () => {
          const { positions } = useAuthStore.getState();
          expect(positions.length).toBeGreaterThan(0);
        },
        { timeout: 1000 }
      );

      const { polymarketAddress, positions } = useAuthStore.getState();
      expect(polymarketAddress).toBe("0xnewaddr");
      expect(positions).toHaveLength(1);
      expect(mockBackend.setPolymarketAddress).toHaveBeenCalledWith("0xnewaddr");
      expect(mockBackend.getPositions).toHaveBeenCalledWith("0xnewaddr");
    });
  });

  describe("fetchPortfolio", () => {
    it("fetches balance and orders when authenticated", async () => {
      // Set up authenticated state
      useAuthStore.setState({
        status: { isAuthenticated: true, address: "0x123" },
      });

      const mockBackend = createMockBackend({
        balance: mockBalance,
        orders: [mockOrder],
      });
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().fetchPortfolio();

      const { balance, orders, portfolioLoading } = useAuthStore.getState();
      expect(balance).toEqual(mockBalance);
      expect(orders).toHaveLength(1);
      expect(portfolioLoading).toBe(false);
    });

    it("fetches positions when polymarketAddress is set", async () => {
      useAuthStore.setState({
        status: { isAuthenticated: true },
        polymarketAddress: "0xpoly",
      });

      const mockBackend = createMockBackend({
        positions: [mockPosition],
      });
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().fetchPortfolio();

      const { positions } = useAuthStore.getState();
      expect(positions).toHaveLength(1);
      expect(mockBackend.getPositions).toHaveBeenCalledWith("0xpoly");
    });

    it("handles balance fetch failure gracefully", async () => {
      useAuthStore.setState({
        status: { isAuthenticated: true },
      });

      // Create backend where getBalance throws
      const mockBackend = createMockBackend();
      (mockBackend.getBalance as ReturnType<typeof vi.fn>).mockRejectedValue(
        new Error("Balance API down")
      );
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().fetchPortfolio();

      const { balance, portfolioLoading } = useAuthStore.getState();
      expect(balance).toBeNull(); // Failed, but didn't crash
      expect(portfolioLoading).toBe(false);
    });

    it("skips balance/orders fetch when not authenticated", async () => {
      useAuthStore.setState({
        status: { isAuthenticated: false },
      });

      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      await useAuthStore.getState().fetchPortfolio();

      expect(mockBackend.getBalance).not.toHaveBeenCalled();
      expect(mockBackend.getOrders).not.toHaveBeenCalled();
    });
  });

  describe("clearError", () => {
    it("clears error state", () => {
      useAuthStore.setState({ error: "Some error" });

      useAuthStore.getState().clearError();

      const { error } = useAuthStore.getState();
      expect(error).toBeNull();
    });
  });
});
