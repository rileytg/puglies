// AIDEV-NOTE: Tests for useBackendEvent hook - subscription and cleanup
import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useBackendEvent, useBackendEvents } from "../useBackendEvent";
import { _setEventSubscriberForTesting, _resetForTesting } from "@/lib/backend";
import { createMockEventSubscriber } from "@/test/mock-adapter";

describe("useBackendEvent", () => {
  let mockSubscriber: ReturnType<typeof createMockEventSubscriber>;

  beforeEach(() => {
    _resetForTesting();
    mockSubscriber = createMockEventSubscriber();
    _setEventSubscriberForTesting(mockSubscriber);
  });

  it("subscribes to event on mount", async () => {
    const callback = vi.fn();

    renderHook(() => useBackendEvent("test_event", callback));

    // Wait for async subscription
    await waitFor(() => {
      expect(mockSubscriber.subscribe).toHaveBeenCalledWith(
        "test_event",
        expect.any(Function)
      );
    });
  });

  it("calls callback when event is emitted", async () => {
    const callback = vi.fn();

    renderHook(() => useBackendEvent("price_update", callback));

    // Wait for subscription to be set up
    await waitFor(() => {
      expect(mockSubscriber.getSubscriptions().has("price_update")).toBe(true);
    });

    // Emit event
    act(() => {
      mockSubscriber.emit("price_update", { price: 0.65 });
    });

    expect(callback).toHaveBeenCalledWith({ price: 0.65 });
  });

  it("unsubscribes on unmount", async () => {
    const callback = vi.fn();

    const { unmount } = renderHook(() => useBackendEvent("test_event", callback));

    // Wait for subscription
    await waitFor(() => {
      expect(mockSubscriber.getSubscriptions().has("test_event")).toBe(true);
    });

    // Unmount
    unmount();

    // Emit after unmount - callback should not be called
    act(() => {
      mockSubscriber.emit("test_event", { data: "after unmount" });
    });

    expect(callback).not.toHaveBeenCalled();
  });

  it("re-subscribes when eventName changes", async () => {
    const callback = vi.fn();

    const { rerender } = renderHook(
      ({ eventName }) => useBackendEvent(eventName, callback),
      { initialProps: { eventName: "event_a" } }
    );

    await waitFor(() => {
      expect(mockSubscriber.getSubscriptions().has("event_a")).toBe(true);
    });

    // Change event name
    rerender({ eventName: "event_b" });

    await waitFor(() => {
      expect(mockSubscriber.getSubscriptions().has("event_b")).toBe(true);
    });

    // Emit to new event
    act(() => {
      mockSubscriber.emit("event_b", { value: "b" });
    });

    expect(callback).toHaveBeenCalledWith({ value: "b" });
  });

  it("uses latest callback (ref pattern)", async () => {
    const callback1 = vi.fn();
    const callback2 = vi.fn();

    const { rerender } = renderHook(
      ({ cb }) => useBackendEvent("event", cb),
      { initialProps: { cb: callback1 } }
    );

    await waitFor(() => {
      expect(mockSubscriber.getSubscriptions().has("event")).toBe(true);
    });

    // Update callback
    rerender({ cb: callback2 });

    // Emit event
    act(() => {
      mockSubscriber.emit("event", { data: "test" });
    });

    // Should call the new callback, not the old one
    expect(callback2).toHaveBeenCalledWith({ data: "test" });
    expect(callback1).not.toHaveBeenCalled();
  });

  it("handles typed payloads", async () => {
    interface PriceUpdate {
      market: string;
      price: number;
    }

    const callback = vi.fn<(payload: PriceUpdate) => void>();

    renderHook(() => useBackendEvent<PriceUpdate>("price_update", callback));

    await waitFor(() => {
      expect(mockSubscriber.getSubscriptions().has("price_update")).toBe(true);
    });

    act(() => {
      mockSubscriber.emit<PriceUpdate>("price_update", {
        market: "0xabc",
        price: 0.75,
      });
    });

    expect(callback).toHaveBeenCalledWith({
      market: "0xabc",
      price: 0.75,
    });
  });
});

describe("useBackendEvents", () => {
  let mockSubscriber: ReturnType<typeof createMockEventSubscriber>;

  beforeEach(() => {
    _resetForTesting();
    mockSubscriber = createMockEventSubscriber();
    _setEventSubscriberForTesting(mockSubscriber);
  });

  it("subscribes to multiple events", async () => {
    const callback1 = vi.fn();
    const callback2 = vi.fn();

    renderHook(() =>
      useBackendEvents([
        { name: "event_a", callback: callback1 },
        { name: "event_b", callback: callback2 },
      ])
    );

    await waitFor(() => {
      expect(mockSubscriber.subscribeMany).toHaveBeenCalled();
    });

    // Emit events
    act(() => {
      mockSubscriber.emit("event_a", { type: "a" });
      mockSubscriber.emit("event_b", { type: "b" });
    });

    expect(callback1).toHaveBeenCalledWith({ type: "a" });
    expect(callback2).toHaveBeenCalledWith({ type: "b" });
  });

  it("unsubscribes all on unmount", async () => {
    const callback1 = vi.fn();
    const callback2 = vi.fn();

    const { unmount } = renderHook(() =>
      useBackendEvents([
        { name: "event_a", callback: callback1 },
        { name: "event_b", callback: callback2 },
      ])
    );

    await waitFor(() => {
      expect(mockSubscriber.subscribeMany).toHaveBeenCalled();
    });

    unmount();

    // Emit after unmount
    act(() => {
      mockSubscriber.emit("event_a", {});
      mockSubscriber.emit("event_b", {});
    });

    expect(callback1).not.toHaveBeenCalled();
    expect(callback2).not.toHaveBeenCalled();
  });

  it("handles empty events array", async () => {
    renderHook(() => useBackendEvents([]));

    await waitFor(() => {
      expect(mockSubscriber.subscribeMany).toHaveBeenCalledWith([]);
    });
  });
});
