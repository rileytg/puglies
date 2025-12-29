import { useEffect, useState, useCallback } from "react";
import { MarketList, MarketSearch } from "@/components/markets";
import { useMarketsStore } from "@/stores/markets";
import { getBackend } from "@/lib/backend";
import { Button } from "@/components/ui/button";
import { RefreshCw } from "lucide-react";

export function Markets() {
  const { markets, isLoading, searchQuery, setMarkets, setLoading, setError, setSearchQuery } =
    useMarketsStore();
  const [debouncedQuery, setDebouncedQuery] = useState(searchQuery);

  // Debounce search query
  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedQuery(searchQuery);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery]);

  const fetchMarkets = useCallback(async () => {
    setLoading(true);
    try {
      const backend = await getBackend();
      const data = debouncedQuery
        ? await backend.searchMarkets(debouncedQuery)
        : await backend.getMarkets(undefined, 50);
      setMarkets(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch markets");
    } finally {
      setLoading(false);
    }
  }, [debouncedQuery, setMarkets, setLoading, setError]);

  useEffect(() => {
    fetchMarkets();
  }, [fetchMarkets]);

  return (
    <div className="flex h-full flex-col gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Markets</h2>
        <Button
          variant="outline"
          size="sm"
          onClick={fetchMarkets}
          disabled={isLoading}
        >
          <RefreshCw className={`mr-2 h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
          Refresh
        </Button>
      </div>

      {/* Search */}
      <div className="max-w-md">
        <MarketSearch value={searchQuery} onChange={setSearchQuery} />
      </div>

      {/* Results */}
      <div className="flex-1 overflow-hidden">
        <MarketList markets={markets} isLoading={isLoading} />
      </div>

      {/* Results count */}
      <div className="text-sm text-muted-foreground">
        {markets.length} markets
      </div>
    </div>
  );
}
