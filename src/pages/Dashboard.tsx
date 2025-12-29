import { useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { MarketList } from "@/components/markets";
import { useMarketsStore } from "@/stores/markets";
import { useAuthStore } from "@/stores/auth";
import { getBackend } from "@/lib/backend";
import { Briefcase, TrendingUp, Activity } from "lucide-react";
import { cn } from "@/lib/utils";

export function Dashboard() {
  const { markets, isLoading, setMarkets, setLoading, setError } = useMarketsStore();
  const { status, balance, positions, orders, fetchPortfolio } = useAuthStore();

  // Check auth status on mount
  useEffect(() => {
    useAuthStore.getState().checkAuthStatus();
  }, []);

  // Fetch portfolio when authenticated
  useEffect(() => {
    if (status.isAuthenticated) {
      fetchPortfolio();
    }
  }, [status.isAuthenticated, fetchPortfolio]);

  // Calculate portfolio value from positions + cash
  const positionValue = positions.reduce((sum, p) => sum + p.currentValue, 0);
  const cashValue = balance ? parseFloat(balance.balance) / 1e6 : 0;
  const portfolioValue = positionValue + cashValue;
  const totalPnl = positions.reduce((sum, p) => sum + p.cashPnl, 0);
  const isProfitable = totalPnl >= 0;

  useEffect(() => {
    async function fetchMarkets() {
      setLoading(true);
      try {
        const backend = await getBackend();
        const data = await backend.getMarkets(undefined, 6);
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

      {/* Stats row */}
      <div className="grid gap-4 sm:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Portfolio Value
            </CardTitle>
            <Briefcase className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            {status.isAuthenticated && (positions.length > 0 || cashValue > 0) ? (
              <>
                <div className="text-2xl font-bold text-green-500">
                  ${portfolioValue.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </div>
                <p className="text-xs text-muted-foreground">Positions + Cash</p>
              </>
            ) : (
              <>
                <div className="text-2xl font-bold">--</div>
                <p className="text-xs text-muted-foreground">
                  {status.isAuthenticated ? "No positions" : "Connect wallet to view"}
                </p>
              </>
            )}
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
            {status.isAuthenticated && positions.length > 0 ? (
              <>
                <div className={cn("text-2xl font-bold", isProfitable ? "text-green-500" : "text-red-500")}>
                  {isProfitable ? "+" : ""}${totalPnl.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </div>
                <p className="text-xs text-muted-foreground">Unrealized</p>
              </>
            ) : (
              <>
                <div className="text-2xl font-bold">--</div>
                <p className="text-xs text-muted-foreground">
                  {status.isAuthenticated ? "No positions" : "Connect wallet to view"}
                </p>
              </>
            )}
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
            {status.isAuthenticated ? (
              <>
                <div className="text-2xl font-bold">{orders.length}</div>
                <p className="text-xs text-muted-foreground">Open orders</p>
              </>
            ) : (
              <>
                <div className="text-2xl font-bold">--</div>
                <p className="text-xs text-muted-foreground">Connect wallet to view</p>
              </>
            )}
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
