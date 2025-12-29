import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { OrderBook } from "@/components/trading/OrderBook";
import { PriceChart, type PriceDataPoint } from "@/components/trading/PriceChart";
import { formatPrice, formatCompactUsd } from "@/lib/utils";
import { getBackend } from "@/lib/backend";
import { useWebSocketStore } from "@/stores/websocket";
import { useOrderBookStore } from "@/stores/orderbook";
import { useBackendEvent } from "@/hooks/useBackendEvent";
import { ArrowLeft, ExternalLink, Clock, DollarSign, Droplets } from "lucide-react";
import { TradeForm } from "@/components/trading";
import { useAuthStore } from "@/stores/auth";
import type { Market, PriceUpdate } from "@/lib/types";
import type { Time } from "lightweight-charts";

export function MarketDetail() {
  // AIDEV-NOTE: marketId is Gamma's internal ID (numeric), not condition_id (hex)
  const { marketId } = useParams<{ marketId: string }>();
  const [market, setMarket] = useState<Market | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [priceHistory, setPriceHistory] = useState<PriceDataPoint[]>([]);
  const [livePrice, setLivePrice] = useState<number | null>(null);

  const { connectToClob, disconnectFromClob, connectToRtds, disconnectFromRtds } =
    useWebSocketStore();
  const getOrderBook = useOrderBookStore((state) => state.getOrderBook);
  const { fetchPortfolio } = useAuthStore();

  // Fetch market data and price history
  useEffect(() => {
    async function fetchMarket() {
      if (!marketId) return;
      setIsLoading(true);
      try {
        const backend = await getBackend();
        const data = await backend.getMarket(marketId);
        setMarket(data);

        // Fetch cached/API price history for the Yes token
        const yesToken = data.tokens.find((t) => t.outcome === "Yes") ?? data.tokens[0];
        if (yesToken) {
          try {
            // AIDEV-NOTE: Fetch price history with caching - interval "max" gets all data
            const historyResult = await backend.getPriceHistory({
              tokenId: yesToken.token_id,
              interval: "max",
              fidelity: 60, // Hourly resolution
            });

            if (historyResult.history.length > 0) {
              // Convert API format {t, p} to chart format {time, value}
              const chartData: PriceDataPoint[] = historyResult.history.map((point) => ({
                time: point.t as Time,
                value: point.p,
              }));
              setPriceHistory(chartData);
              console.log(
                `[MarketDetail] Loaded ${historyResult.history.length} price points ` +
                `(cached: ${historyResult.cachedCount}, fetched: ${historyResult.fetchedCount})`
              );
            } else {
              // Fallback: just use current price
              const now = Math.floor(Date.now() / 1000) as Time;
              setPriceHistory([{ time: now, value: yesToken.price }]);
            }
          } catch (histErr) {
            console.warn("[MarketDetail] Failed to fetch price history:", histErr);
            // Fallback to current price
            const now = Math.floor(Date.now() / 1000) as Time;
            setPriceHistory([{ time: now, value: yesToken.price }]);
          }
        }
      } catch (err) {
        console.error("[MarketDetail] Error fetching market:", err);
        setError(err instanceof Error ? err.message : "Failed to fetch market");
      } finally {
        setIsLoading(false);
      }
    }
    fetchMarket();
  }, [marketId]);

  // Connect to WebSocket when market is loaded
  // AIDEV-NOTE: Both CLOB and RTDS use token IDs (not condition_id)
  useEffect(() => {
    if (!market) return;

    const tokenIds = market.tokens.map((t) => t.token_id);
    connectToClob(tokenIds);
    connectToRtds(tokenIds); // RTDS clob_market topic needs token IDs

    return () => {
      disconnectFromClob();
      disconnectFromRtds();
    };
  }, [market, connectToClob, disconnectFromClob, connectToRtds, disconnectFromRtds]);

  // Listen for price updates from CLOB WebSocket
  // AIDEV-NOTE: Price updates have market (condition_id) and asset_id (token_id)
  const yesTokenIdRef = market?.tokens.find((t) => t.outcome === "Yes")?.token_id;
  useBackendEvent<PriceUpdate>("price_update", (update) => {
    // Match by market condition_id OR by Yes token asset_id
    const isMatch = update.market === market?.condition_id ||
                    update.asset_id === yesTokenIdRef;
    if (isMatch && update.asset_id === yesTokenIdRef) {
      setLivePrice(update.price);
      const time = (update.timestamp
        ? update.timestamp / 1000
        : Date.now() / 1000) as Time;
      setPriceHistory((prev) => [...prev.slice(-999), { time, value: update.price }]);
    }
  });

  // Get order book for the Yes token
  const yesTokenId = market?.tokens.find((t) => t.outcome === "Yes")?.token_id;
  const orderBookData = yesTokenId ? getOrderBook(yesTokenId) : undefined;

  if (isLoading) {
    return (
      <div className="space-y-4">
        <div className="h-8 w-48 animate-pulse rounded bg-card" />
        <div className="h-64 animate-pulse rounded-xl bg-card" />
      </div>
    );
  }

  if (error || !market) {
    return (
      <div className="space-y-4">
        <Link to="/markets">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to Markets
          </Button>
        </Link>
        <Card>
          <CardContent className="py-8 text-center text-muted-foreground">
            {error || "Market not found"}
          </CardContent>
        </Card>
      </div>
    );
  }

  const yesToken = market.tokens.find((t) => t.outcome === "Yes") ?? market.tokens[0];
  const noToken = market.tokens.find((t) => t.outcome === "No") ?? market.tokens[1];
  const yesPrice = livePrice ?? yesToken?.price ?? 0;
  const noPrice = noToken?.price ?? 1 - yesPrice;

  return (
    <div className="space-y-6">
      {/* Back button */}
      <Link to="/markets">
        <Button variant="ghost" size="sm">
          <ArrowLeft className="mr-2 h-4 w-4" />
          Back to Markets
        </Button>
      </Link>

      {/* Header */}
      <div className="flex gap-4">
        {market.image && (
          <img
            src={market.image}
            alt=""
            className="h-20 w-20 shrink-0 rounded-xl object-cover"
          />
        )}
        <div className="flex-1">
          <h1 className="text-2xl font-bold">{market.question}</h1>
          {market.tags && market.tags.length > 0 && (
            <div className="mt-2 flex flex-wrap gap-1">
              {market.tags.map((tag) => (
                <Badge key={tag} variant="secondary">
                  {tag}
                </Badge>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Stats */}
      <div className="grid gap-4 sm:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
              <DollarSign className="h-4 w-4" />
              Volume
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xl font-bold">
              {formatCompactUsd(market.volume_num)}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
              <Droplets className="h-4 w-4" />
              Liquidity
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xl font-bold">
              {formatCompactUsd(market.liquidity_num)}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
              <Clock className="h-4 w-4" />
              End Date
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xl font-bold">
              {market.end_date_iso
                ? new Date(market.end_date_iso).toLocaleDateString()
                : "TBD"}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-muted-foreground">
              Spread
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-xl font-bold">
              {(market.spread * 100).toFixed(1)}%
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Prices */}
      <Card>
        <CardHeader>
          <CardTitle>Current Prices</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-8">
            <div className="flex-1">
              <div className="mb-2 text-sm text-muted-foreground">Yes</div>
              <div className="text-4xl font-bold text-green">
                {formatPrice(yesPrice)}
              </div>
              <div className="mt-1 text-sm text-muted-foreground">
                Implied probability: {(yesPrice * 100).toFixed(1)}%
              </div>
            </div>
            <Separator orientation="vertical" className="h-24" />
            <div className="flex-1">
              <div className="mb-2 text-sm text-muted-foreground">No</div>
              <div className="text-4xl font-bold text-red">
                {formatPrice(noPrice)}
              </div>
              <div className="mt-1 text-sm text-muted-foreground">
                Implied probability: {(noPrice * 100).toFixed(1)}%
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Description */}
      {market.description && (
        <Card>
          <CardHeader>
            <CardTitle>Description</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="whitespace-pre-wrap text-muted-foreground">
              {market.description}
            </p>
          </CardContent>
        </Card>
      )}

      {/* Chart and Order Book */}
      <div className="grid gap-4 lg:grid-cols-3">
        {/* Price Chart */}
        <Card className="lg:col-span-2">
          <CardHeader>
            <CardTitle>Price History</CardTitle>
          </CardHeader>
          <CardContent>
            {priceHistory.length > 0 ? (
              <PriceChart data={priceHistory} height={300} />
            ) : (
              <div className="flex h-[300px] items-center justify-center text-muted-foreground">
                Waiting for price data...
              </div>
            )}
          </CardContent>
        </Card>

        {/* Order Book */}
        <Card>
          <CardHeader>
            <CardTitle>Order Book (Yes)</CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            {orderBookData ? (
              <OrderBook
                bids={orderBookData.bids}
                asks={orderBookData.asks}
                maxLevels={8}
                className="h-[300px]"
              />
            ) : (
              <div className="flex h-[300px] items-center justify-center text-muted-foreground">
                Connecting to order book...
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Trading Forms */}
      <div className="grid gap-4 md:grid-cols-2">
        {yesToken && (
          <TradeForm
            market={market}
            token={yesToken}
            onOrderPlaced={() => fetchPortfolio()}
          />
        )}
        {noToken && (
          <TradeForm
            market={market}
            token={noToken}
            onOrderPlaced={() => fetchPortfolio()}
          />
        )}
      </div>

      {/* External link */}
      <div>
        <Button variant="outline" asChild>
          <a
            href={`https://polymarket.com/event/${market.market_slug}`}
            target="_blank"
            rel="noopener noreferrer"
          >
            View on Polymarket
            <ExternalLink className="ml-2 h-4 w-4" />
          </a>
        </Button>
      </div>
    </div>
  );
}
