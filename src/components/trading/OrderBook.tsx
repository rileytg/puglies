// AIDEV-NOTE: Order book display with bid/ask depth visualization
import { useMemo } from "react";
import { cn } from "@/lib/utils";
import type { OrderBookLevel } from "@/lib/types";

interface OrderBookProps {
  bids: OrderBookLevel[];
  asks: OrderBookLevel[];
  maxLevels?: number;
  className?: string;
}

export function OrderBook({
  bids,
  asks,
  maxLevels = 10,
  className,
}: OrderBookProps) {
  // Calculate max size for depth bar scaling
  const { maxBidSize, maxAskSize, spread } = useMemo(() => {
    const bidSizes = bids.slice(0, maxLevels).map((b) => parseFloat(b.size));
    const askSizes = asks.slice(0, maxLevels).map((a) => parseFloat(a.size));

    const topBid = bids[0] ? parseFloat(bids[0].price) : 0;
    const topAsk = asks[0] ? parseFloat(asks[0].price) : 0;
    const spreadVal = topAsk > 0 && topBid > 0 ? topAsk - topBid : null;

    return {
      maxBidSize: Math.max(...bidSizes, 0),
      maxAskSize: Math.max(...askSizes, 0),
      spread: spreadVal,
    };
  }, [bids, asks, maxLevels]);

  const displayedBids = bids.slice(0, maxLevels);
  const displayedAsks = asks.slice(0, maxLevels);

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Header */}
      <div className="grid grid-cols-3 text-xs text-muted-foreground px-2 py-1 border-b">
        <span>Price</span>
        <span className="text-right">Size</span>
        <span className="text-right">Total</span>
      </div>

      {/* Asks (sell orders) - reversed to show lowest at bottom */}
      <div className="flex-1 overflow-hidden">
        <div className="flex flex-col-reverse">
          {displayedAsks.map((level, idx) => (
            <OrderBookRow
              key={`ask-${level.price}`}
              level={level}
              side="ask"
              maxSize={maxAskSize}
              runningTotal={calculateRunningTotal(displayedAsks, idx)}
            />
          ))}
        </div>
      </div>

      {/* Spread indicator */}
      <div className="flex items-center justify-center py-1 border-y bg-muted/50">
        <span className="text-xs text-muted-foreground">
          Spread:{" "}
          {spread !== null ? (
            <span className="font-mono">{(spread * 100).toFixed(2)}¢</span>
          ) : (
            "—"
          )}
        </span>
      </div>

      {/* Bids (buy orders) */}
      <div className="flex-1 overflow-hidden">
        <div className="flex flex-col">
          {displayedBids.map((level, idx) => (
            <OrderBookRow
              key={`bid-${level.price}`}
              level={level}
              side="bid"
              maxSize={maxBidSize}
              runningTotal={calculateRunningTotal(displayedBids, idx)}
            />
          ))}
        </div>
      </div>
    </div>
  );
}

interface OrderBookRowProps {
  level: OrderBookLevel;
  side: "bid" | "ask";
  maxSize: number;
  runningTotal: number;
}

function OrderBookRow({ level, side, maxSize, runningTotal }: OrderBookRowProps) {
  const size = parseFloat(level.size);
  const price = parseFloat(level.price);
  const depthPercent = maxSize > 0 ? (size / maxSize) * 100 : 0;

  return (
    <div className="relative grid grid-cols-3 text-xs px-2 py-0.5 hover:bg-muted/50">
      {/* Depth bar background */}
      <div
        className={cn(
          "absolute inset-y-0 right-0 opacity-20",
          side === "bid" ? "bg-green-500" : "bg-red-500"
        )}
        style={{ width: `${depthPercent}%` }}
      />

      {/* Content */}
      <span
        className={cn(
          "font-mono relative",
          side === "bid" ? "text-green-600" : "text-red-600"
        )}
      >
        {(price * 100).toFixed(1)}¢
      </span>
      <span className="font-mono text-right relative">{formatSize(size)}</span>
      <span className="font-mono text-right text-muted-foreground relative">
        {formatSize(runningTotal)}
      </span>
    </div>
  );
}

function calculateRunningTotal(levels: OrderBookLevel[], upToIndex: number): number {
  return levels
    .slice(0, upToIndex + 1)
    .reduce((sum, level) => sum + parseFloat(level.size), 0);
}

function formatSize(size: number): string {
  if (size >= 1000000) {
    return `${(size / 1000000).toFixed(1)}M`;
  }
  if (size >= 1000) {
    return `${(size / 1000).toFixed(1)}K`;
  }
  return size.toFixed(0);
}
