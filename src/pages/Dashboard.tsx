import { useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MarketList } from "@/components/markets";
import { useMarketsStore } from "@/stores/markets";
import { getMarkets } from "@/lib/tauri";
import { Briefcase, TrendingUp, Activity } from "lucide-react";

export function Dashboard() {
  const { markets, isLoading, setMarkets, setLoading, setError } = useMarketsStore();

  useEffect(() => {
    async function fetchMarkets() {
      setLoading(true);
      try {
        const data = await getMarkets(undefined, 6);
        setMarkets(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to fetch markets");
      } finally {
        setLoading(false);
      }
    }
    fetchMarkets();
  }, [setMarkets, setLoading, setError]);

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Dashboard</h2>

      {/* Stats row (placeholder for Phase 3) */}
      <div className="grid gap-4 sm:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Portfolio Value
            </CardTitle>
            <Briefcase className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">--</div>
            <p className="text-xs text-muted-foreground">Connect wallet to view</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Open P&L
            </CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">--</div>
            <p className="text-xs text-muted-foreground">Connect wallet to view</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Active Orders
            </CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">--</div>
            <p className="text-xs text-muted-foreground">Connect wallet to view</p>
          </CardContent>
        </Card>
      </div>

      {/* Top markets */}
      <div>
        <h3 className="mb-4 text-lg font-semibold">Top Markets</h3>
        <MarketList markets={markets.slice(0, 6)} isLoading={isLoading} />
      </div>
    </div>
  );
}
