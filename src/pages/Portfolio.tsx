// AIDEV-NOTE: Portfolio page - displays positions, orders, and balance

import { useEffect, useState } from "react";
import { useAuthStore } from "@/stores/auth";
import { useMarketsStore } from "@/stores/markets";
import { useTradingStore } from "@/stores/trading";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Badge } from "@/components/ui/badge";
import { Loader2, RefreshCw, TrendingUp, TrendingDown, Wallet, Search, X, XCircle } from "lucide-react";
import { cn } from "@/lib/utils";

export function Portfolio() {
  const {
    status,
    balance,
    positions,
    orders,
    portfolioLoading,
    polymarketAddress,
    fetchPortfolio,
    setPolymarketAddress,
  } = useAuthStore();

  const [addressInput, setAddressInput] = useState("");

  // Sync address input with stored value
  useEffect(() => {
    if (polymarketAddress && !addressInput) {
      setAddressInput(polymarketAddress);
    }
  }, [polymarketAddress]);

  const { markets } = useMarketsStore();
  const { cancelOrder, cancelAllOrders, isCancelling } = useTradingStore();

  // Check auth status on mount (in case store was reset by HMR)
  useEffect(() => {
    useAuthStore.getState().checkAuthStatus();
  }, []);

  // Refresh portfolio data periodically
  useEffect(() => {
    if (status.isAuthenticated) {
      fetchPortfolio();

      // Refresh every 30 seconds
      const interval = setInterval(fetchPortfolio, 30000);
      return () => clearInterval(interval);
    }
  }, [status.isAuthenticated, fetchPortfolio]);

  // Get market question for a condition ID
  const getMarketQuestion = (conditionId: string) => {
    const market = markets.find((m) => m.condition_id === conditionId);
    return market?.question || conditionId.slice(0, 8) + "...";
  };

  if (!status.isAuthenticated) {
    return (
      <div className="flex h-full items-center justify-center">
        <Card className="w-96">
          <CardContent className="pt-6 text-center">
            <Wallet className="mx-auto h-12 w-12 text-muted-foreground" />
            <h2 className="mt-4 text-lg font-semibold">Connect Wallet</h2>
            <p className="mt-2 text-sm text-muted-foreground">
              Connect your wallet to view your portfolio, positions, and orders.
            </p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">Portfolio</h1>
        <Button
          variant="outline"
          size="sm"
          onClick={() => fetchPortfolio()}
          disabled={portfolioLoading}
        >
          {portfolioLoading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="mr-2 h-4 w-4" />
          )}
          Refresh
        </Button>
      </div>

      {/* Polymarket Address Input */}
      <Card>
        <CardHeader>
          <CardTitle className="text-sm font-medium text-muted-foreground">
            Polymarket Address
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <div className="flex-1">
              <Input
                placeholder="Enter your Polymarket wallet address (0x...)"
                value={addressInput}
                onChange={(e) => setAddressInput(e.target.value)}
              />
            </div>
            <Button
              onClick={() => setPolymarketAddress(addressInput)}
              disabled={!addressInput || portfolioLoading}
            >
              <Search className="mr-2 h-4 w-4" />
              Load Positions
            </Button>
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            Your Polymarket address may differ from your signing wallet. Check your Polymarket profile for the correct address.
          </p>
        </CardContent>
      </Card>

      {/* Summary Cards */}
      {(() => {
        const positionValue = positions.reduce((sum, p) => sum + p.currentValue, 0);
        const cashValue = balance ? parseFloat(balance.balance) / 1e6 : 0;
        const totalValue = positionValue + cashValue;
        const totalPnl = positions.reduce((sum, p) => sum + p.cashPnl, 0);
        const isProfitable = totalPnl >= 0;

        return (
          <div className="grid gap-4 sm:grid-cols-3">
            {/* Total Portfolio Value Card */}
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  Portfolio
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold text-green-500">
                  ${totalValue.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </div>
                <p className="mt-1 text-sm text-muted-foreground">
                  Positions + Cash
                </p>
              </CardContent>
            </Card>

            {/* Cash Balance Card */}
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  Cash
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-3xl font-bold text-green-500">
                  ${cashValue.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </div>
                <p className="mt-1 text-sm text-muted-foreground">
                  Available USDC
                </p>
              </CardContent>
            </Card>

            {/* Total P&L Card */}
            <Card>
              <CardHeader>
                <CardTitle className="text-sm font-medium text-muted-foreground">
                  Unrealized P&L
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className={cn("text-3xl font-bold", isProfitable ? "text-green-500" : "text-red-500")}>
                  {isProfitable ? "+" : ""}${totalPnl.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
                </div>
                <p className="mt-1 text-sm text-muted-foreground flex items-center gap-1">
                  {isProfitable ? <TrendingUp className="h-3 w-3" /> : <TrendingDown className="h-3 w-3" />}
                  {positions.length} positions
                </p>
              </CardContent>
            </Card>
          </div>
        );
      })()}

      {/* Positions */}
      <Card>
        <CardHeader>
          <CardTitle>Positions</CardTitle>
        </CardHeader>
        <CardContent>
          {!polymarketAddress ? (
            <p className="text-sm text-muted-foreground">
              Enter your Polymarket address above to view positions
            </p>
          ) : positions.length === 0 ? (
            <p className="text-sm text-muted-foreground">No open positions</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Market</TableHead>
                  <TableHead>Outcome</TableHead>
                  <TableHead className="text-right">Size</TableHead>
                  <TableHead className="text-right">Avg Price</TableHead>
                  <TableHead className="text-right">Current</TableHead>
                  <TableHead className="text-right">P&L</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {positions.map((position, idx) => {
                  const isProfitable = position.cashPnl >= 0;

                  return (
                    <TableRow key={position.asset || idx}>
                      <TableCell className="max-w-[200px] truncate">
                        {position.title || getMarketQuestion(position.conditionId)}
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline">{position.outcome}</Badge>
                      </TableCell>
                      <TableCell className="text-right font-mono">
                        {position.size.toFixed(2)}
                      </TableCell>
                      <TableCell className="text-right font-mono">
                        {(position.avgPrice * 100).toFixed(1)}¢
                      </TableCell>
                      <TableCell className="text-right font-mono">
                        {(position.curPrice * 100).toFixed(1)}¢
                      </TableCell>
                      <TableCell
                        className={cn(
                          "text-right font-mono",
                          isProfitable ? "text-green-500" : "text-red-500"
                        )}
                      >
                        <span className="flex items-center justify-end gap-1">
                          {isProfitable ? (
                            <TrendingUp className="h-3 w-3" />
                          ) : (
                            <TrendingDown className="h-3 w-3" />
                          )}
                          ${position.cashPnl.toFixed(2)} ({position.percentPnl.toFixed(1)}%)
                        </span>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Open Orders */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>Open Orders</CardTitle>
          {orders.length > 0 && (
            <Button
              variant="destructive"
              size="sm"
              onClick={async () => {
                await cancelAllOrders();
                fetchPortfolio();
              }}
              disabled={isCancelling}
            >
              {isCancelling ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <XCircle className="mr-2 h-4 w-4" />
              )}
              Cancel All
            </Button>
          )}
        </CardHeader>
        <CardContent>
          {orders.length === 0 ? (
            <p className="text-sm text-muted-foreground">No open orders</p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Market</TableHead>
                  <TableHead>Side</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead className="text-right">Price</TableHead>
                  <TableHead className="text-right">Size</TableHead>
                  <TableHead className="text-right">Filled</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead className="w-[50px]"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {orders.map((order) => (
                  <TableRow key={order.id}>
                    <TableCell className="max-w-[200px] truncate font-mono text-xs">
                      {order.market || order.asset.slice(0, 8) + "..."}
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={order.side === "BUY" ? "default" : "destructive"}
                      >
                        {order.side}
                      </Badge>
                    </TableCell>
                    <TableCell className="text-xs">{order.orderType}</TableCell>
                    <TableCell className="text-right font-mono">
                      {(parseFloat(order.price) * 100).toFixed(1)}¢
                    </TableCell>
                    <TableCell className="text-right font-mono">
                      {parseFloat(order.originalSize).toFixed(2)}
                    </TableCell>
                    <TableCell className="text-right font-mono">
                      {parseFloat(order.sizeMatched).toFixed(2)}
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="text-xs">
                        {order.status}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-muted-foreground hover:text-destructive"
                        onClick={async () => {
                          await cancelOrder(order.id);
                          fetchPortfolio();
                        }}
                        disabled={isCancelling}
                      >
                        <X className="h-4 w-4" />
                      </Button>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
