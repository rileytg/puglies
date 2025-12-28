import { Link } from "react-router-dom";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { formatPrice, formatCompactUsd } from "@/lib/utils";
import type { Market } from "@/lib/types";

interface MarketCardProps {
  market: Market;
}

export function MarketCard({ market }: MarketCardProps) {
  // Get YES token price (first token is typically YES)
  const yesToken = market.tokens.find((t) => t.outcome === "Yes") ?? market.tokens[0];
  const yesPrice = yesToken?.price ?? 0;

  return (
    <Link to={`/markets/${market.condition_id}`}>
      <Card className="transition-colors hover:border-primary/50 hover:bg-card/80">
        <CardHeader className="pb-2">
          <div className="flex items-start justify-between gap-2">
            <CardTitle className="line-clamp-2 text-sm font-medium leading-snug">
              {market.question}
            </CardTitle>
            {market.image && (
              <img
                src={market.image}
                alt=""
                className="h-10 w-10 shrink-0 rounded-md object-cover"
              />
            )}
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            {/* Price display */}
            <div className="flex items-center gap-3">
              <div className="flex flex-col">
                <span className="text-xs text-muted-foreground">Yes</span>
                <span className="text-lg font-bold text-green">
                  {formatPrice(yesPrice)}
                </span>
              </div>
              <div className="flex flex-col">
                <span className="text-xs text-muted-foreground">No</span>
                <span className="text-lg font-bold text-red">
                  {formatPrice(1 - yesPrice)}
                </span>
              </div>
            </div>

            {/* Volume */}
            <div className="text-right">
              <span className="text-xs text-muted-foreground">Volume</span>
              <div className="font-medium">{formatCompactUsd(market.volume_num)}</div>
            </div>
          </div>

          {/* Tags */}
          {market.tags && market.tags.length > 0 && (
            <div className="mt-3 flex flex-wrap gap-1">
              {market.tags.slice(0, 3).map((tag) => (
                <Badge key={tag} variant="secondary" className="text-xs">
                  {tag}
                </Badge>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </Link>
  );
}
