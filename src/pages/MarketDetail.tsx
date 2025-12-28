import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { formatPrice, formatCompactUsd } from "@/lib/utils";
import { getMarket } from "@/lib/tauri";
import { ArrowLeft, ExternalLink, Clock, DollarSign, Droplets } from "lucide-react";
import type { Market } from "@/lib/types";

export function MarketDetail() {
  const { conditionId } = useParams<{ conditionId: string }>();
  const [market, setMarket] = useState<Market | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchMarket() {
      if (!conditionId) return;
      setIsLoading(true);
      try {
        const data = await getMarket(conditionId);
        setMarket(data);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to fetch market");
      } finally {
        setIsLoading(false);
      }
    }
    fetchMarket();
  }, [conditionId]);

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
  const yesPrice = yesToken?.price ?? 0;
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

      {/* Trading placeholder */}
      <Card className="border-dashed">
        <CardContent className="py-8 text-center">
          <p className="text-muted-foreground">
            Trading will be available after authentication (Phase 4)
          </p>
        </CardContent>
      </Card>

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
