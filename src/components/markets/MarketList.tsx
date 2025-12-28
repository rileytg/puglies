import { MarketCard } from "./MarketCard";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { Market } from "@/lib/types";

interface MarketListProps {
  markets: Market[];
  isLoading?: boolean;
}

export function MarketList({ markets, isLoading }: MarketListProps) {
  if (isLoading) {
    return (
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {Array.from({ length: 6 }).map((_, i) => (
          <div
            key={i}
            className="h-40 animate-pulse rounded-xl border border-border bg-card"
          />
        ))}
      </div>
    );
  }

  if (markets.length === 0) {
    return (
      <div className="flex h-40 items-center justify-center text-muted-foreground">
        No markets found
      </div>
    );
  }

  return (
    <ScrollArea className="h-full">
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
        {markets.map((market) => (
          <MarketCard key={market.condition_id} market={market} />
        ))}
      </div>
    </ScrollArea>
  );
}
