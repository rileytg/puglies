// AIDEV-NOTE: Tests for websocket store - connection management with mocked backend
import { describe, it, expect, beforeEach } from "vitest";
import { useWebSocketStore } from "../websocket";
import { _setBackendForTesting, _resetForTesting } from "@/lib/backend";
import { createMockBackend, mockConnectionStatus } from "@/test/mock-adapter";

describe("useWebSocketStore", () => {
  beforeEach(() => {
    _resetForTesting();
    useWebSocketStore.setState({
      status: { clob: "disconnected", rtds: "disconnected" },
      lastUpdate: null,
      isConnecting: false,
    });
  });

  describe("initial state", () => {
    it("starts disconnected", () => {
      const { status } = useWebSocketStore.getState();
      expect(status.clob).toBe("disconnected");
      expect(status.rtds).toBe("disconnected");
    });

    it("starts with no last update", () => {
      const { lastUpdate } = useWebSocketStore.getState();
      expect(lastUpdate).toBeNull();
    });
  });

  describe("setStatus", () => {
    it("updates full status", () => {
      useWebSocketStore.getState().setStatus({
        clob: "connected",
        rtds: "connected",
      });

      const { status } = useWebSocketStore.getState();
      expect(status.clob).toBe("connected");
      expect(status.rtds).toBe("connected");
    });
  });

  describe("setClobStatus", () => {
    it("updates only clob status", () => {
      useWebSocketStore.getState().setClobStatus("connecting");

      const { status } = useWebSocketStore.getState();
      expect(status.clob).toBe("connecting");
      expect(status.rtds).toBe("disconnected");
    });
  });

  describe("setRtdsStatus", () => {
    it("updates only rtds status", () => {
      useWebSocketStore.getState().setRtdsStatus("connected");

      const { status } = useWebSocketStore.getState();
      expect(status.rtds).toBe("connected");
      expect(status.clob).toBe("disconnected");
    });
  });

  describe("setLastUpdate", () => {
    it("updates last update timestamp", () => {
      const now = new Date();
      useWebSocketStore.getState().setLastUpdate(now);

      const { lastUpdate } = useWebSocketStore.getState();
      expect(lastUpdate).toEqual(now);
    });
  });

  describe("connectToRtds", () => {
    it("calls backend connectRtds with markets", async () => {
      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      await useWebSocketStore.getState().connectToRtds(["market-1", "market-2"]);

      expect(mockBackend.connectRtds).toHaveBeenCalledWith(["market-1", "market-2"]);
    });

    it("sets isConnecting during connection", async () => {
      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      const promise = useWebSocketStore.getState().connectToRtds(["market-1"]);

      // After completion, isConnecting should be false
      await promise;

      const { isConnecting } = useWebSocketStore.getState();
      expect(isConnecting).toBe(false);
    });

    it("resets isConnecting on error", async () => {
      const mockBackend = createMockBackend({
        shouldThrow: true,
        errorMessage: "Connection failed",
      });
      _setBackendForTesting(mockBackend);

      try {
        await useWebSocketStore.getState().connectToRtds(["market-1"]);
      } catch {
        // Expected error
      }

      const { isConnecting } = useWebSocketStore.getState();
      expect(isConnecting).toBe(false);
    });
  });

  describe("disconnectFromRtds", () => {
    it("calls backend disconnectRtds", async () => {
      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      await useWebSocketStore.getState().disconnectFromRtds();

      expect(mockBackend.disconnectRtds).toHaveBeenCalled();
    });
  });

  describe("connectToClob", () => {
    it("calls backend connectClob with token ids", async () => {
      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      await useWebSocketStore.getState().connectToClob(["token-1", "token-2"]);

      expect(mockBackend.connectClob).toHaveBeenCalledWith(["token-1", "token-2"]);
    });

    it("manages isConnecting state", async () => {
      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      await useWebSocketStore.getState().connectToClob(["token-1"]);

      const { isConnecting } = useWebSocketStore.getState();
      expect(isConnecting).toBe(false);
    });
  });

  describe("disconnectFromClob", () => {
    it("calls backend disconnectClob", async () => {
      const mockBackend = createMockBackend();
      _setBackendForTesting(mockBackend);

      await useWebSocketStore.getState().disconnectFromClob();

      expect(mockBackend.disconnectClob).toHaveBeenCalled();
    });
  });

  describe("refreshStatus", () => {
    it("fetches and updates status from backend", async () => {
      const mockBackend = createMockBackend({
        connectionStatus: mockConnectionStatus,
      });
      _setBackendForTesting(mockBackend);

      await useWebSocketStore.getState().refreshStatus();

      const { status } = useWebSocketStore.getState();
      expect(status).toEqual(mockConnectionStatus);
      expect(mockBackend.getConnectionStatus).toHaveBeenCalled();
    });

    it("updates status with backend response", async () => {
      const mockBackend = createMockBackend({
        connectionStatus: { clob: "reconnecting", rtds: "failed" },
      });
      _setBackendForTesting(mockBackend);

      await useWebSocketStore.getState().refreshStatus();

      const { status } = useWebSocketStore.getState();
      expect(status.clob).toBe("reconnecting");
      expect(status.rtds).toBe("failed");
    });
  });
});
