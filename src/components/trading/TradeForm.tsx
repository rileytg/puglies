// AIDEV-NOTE: Trading form for placing buy/sell orders on a market

import { useState, useCallback } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useAuthStore } from "@/stores/auth";
import { useTradingStore } from "@/stores/trading";
import { Loader2, AlertCircle, CheckCircle2 } from "lucide-react";
import type { Market, OrderSide, OrderTimeInForce, Token } from "@/lib/types";
import { cn } from "@/lib/utils";

interface TradeFormProps {
  market: Market;
  token: Token;
  onOrderPlaced?: () => void;
}

export function TradeForm({ market, token, onOrderPlaced }: TradeFormProps) {
  const [side, setSide] = useState<OrderSide>("Buy");
  const [price, setPrice] = useState((token.price || 0.5).toFixed(2));
  const [size, setSize] = useState("");
  const [privateKey, setPrivateKey] = useState("");
  const [showKeyInput, setShowKeyInput] = useState(false);

  const { status, balance } = useAuthStore();
  const {
    placeOrder,
    isPlacingOrder,
    orderError,
    lastOrderResult,
    clearErrors,
    clearOrderResult,
  } = useTradingStore();

  const parsedPrice = parseFloat(price) || 0;
  const parsedSize = parseFloat(size) || 0;
  const totalCost = parsedPrice * parsedSize;

  // AIDEV-NOTE: Balance is in wei (6 decimals), convert to dollars
  const availableBalance = balance
    ? parseFloat(balance.balance) / 1_000_000
    : 0;

  const canSubmit =
    status.isAuthenticated &&
    parsedPrice > 0 &&
    parsedPrice < 1 &&
    parsedSize > 0 &&
    parsedSize >= market.minimum_order_size &&
    (side === "Sell" || totalCost <= availableBalance) &&
    privateKey.length > 0;

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!canSubmit) return;

      clearErrors();
      clearOrderResult();

      const success = await placeOrder(
        {
          tokenId: token.token_id,
          side,
          price: parsedPrice,
          size: parsedSize,
          orderType: "Gtc" as OrderTimeInForce,
        },
        privateKey
      );

      if (success) {
        setSize("");
        setPrivateKey("");
        setShowKeyInput(false);
        onOrderPlaced?.();
      }
    },
    [
      canSubmit,
      placeOrder,
      token.token_id,
      side,
      parsedPrice,
      parsedSize,
      privateKey,
      onOrderPlaced,
      clearErrors,
      clearOrderResult,
    ]
  );

  if (!status.isAuthenticated) {
    return (
      <Card className="border-dashed">
        <CardContent className="py-6 text-center text-muted-foreground">
          Connect your wallet to trade
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="text-lg">Trade: {token.outcome}</CardTitle>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Buy/Sell tabs */}
          <Tabs value={side} onValueChange={(v) => setSide(v as OrderSide)}>
            <TabsList className="grid w-full grid-cols-2">
              <TabsTrigger
                value="Buy"
                className={cn(
                  "data-[state=active]:bg-green-600 data-[state=active]:text-white"
                )}
              >
                Buy
              </TabsTrigger>
              <TabsTrigger
                value="Sell"
                className={cn(
                  "data-[state=active]:bg-red-600 data-[state=active]:text-white"
                )}
              >
                Sell
              </TabsTrigger>
            </TabsList>
          </Tabs>

          {/* Price input */}
          <div className="space-y-2">
            <Label htmlFor="price">Limit Price</Label>
            <div className="relative">
              <Input
                id="price"
                type="number"
                step="0.01"
                min="0.01"
                max="0.99"
                placeholder="0.50"
                value={price}
                onChange={(e) => setPrice(e.target.value)}
                className="pr-8"
                disabled={isPlacingOrder}
              />
              <span className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground text-sm">
                ¢
              </span>
            </div>
            <p className="text-xs text-muted-foreground">
              Current: {((token.price || 0) * 100).toFixed(1)}¢
            </p>
          </div>

          {/* Size input */}
          <div className="space-y-2">
            <Label htmlFor="size">Shares</Label>
            <Input
              id="size"
              type="number"
              step="1"
              min={market.minimum_order_size}
              placeholder={`Min: ${market.minimum_order_size}`}
              value={size}
              onChange={(e) => setSize(e.target.value)}
              disabled={isPlacingOrder}
            />
          </div>

          {/* Order summary */}
          <div className="rounded-lg bg-muted p-3 text-sm space-y-1">
            <div className="flex justify-between">
              <span className="text-muted-foreground">
                {side === "Buy" ? "Total Cost" : "Total Return"}
              </span>
              <span className="font-mono font-medium">
                ${totalCost.toFixed(2)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Available</span>
              <span className="font-mono text-muted-foreground">
                ${availableBalance.toFixed(2)}
              </span>
            </div>
            {side === "Buy" && totalCost > availableBalance && (
              <p className="text-xs text-red-500 mt-1">Insufficient balance</p>
            )}
          </div>

          {/* Private key input (required for signing) */}
          {!showKeyInput ? (
            <Button
              type="button"
              variant="outline"
              className="w-full"
              onClick={() => setShowKeyInput(true)}
              disabled={isPlacingOrder}
            >
              Enter key to sign order
            </Button>
          ) : (
            <div className="space-y-2">
              <Label htmlFor="privateKey">Private Key (for signing)</Label>
              <Input
                id="privateKey"
                type="password"
                placeholder="0x..."
                value={privateKey}
                onChange={(e) => setPrivateKey(e.target.value)}
                className="font-mono"
                disabled={isPlacingOrder}
              />
              <p className="text-xs text-muted-foreground">
                Required to sign the order. Not stored.
              </p>
            </div>
          )}

          {/* Success message */}
          {lastOrderResult?.success && (
            <Alert className="border-green-500 bg-green-500/10">
              <CheckCircle2 className="h-4 w-4 text-green-500" />
              <AlertDescription className="text-green-500">
                Order placed! Status: {lastOrderResult.status || "pending"}
              </AlertDescription>
            </Alert>
          )}

          {/* Error display */}
          {orderError && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{orderError}</AlertDescription>
            </Alert>
          )}

          {/* Submit button */}
          <Button
            type="submit"
            className={cn(
              "w-full",
              side === "Buy"
                ? "bg-green-600 hover:bg-green-700"
                : "bg-red-600 hover:bg-red-700"
            )}
            disabled={!canSubmit || isPlacingOrder}
          >
            {isPlacingOrder ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Placing Order...
              </>
            ) : (
              `${side} ${parsedSize || 0} @ ${(parsedPrice * 100).toFixed(0)}¢`
            )}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
