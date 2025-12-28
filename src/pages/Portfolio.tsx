import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Wallet } from "lucide-react";

export function Portfolio() {
  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Portfolio</h2>

      <Card className="border-dashed">
        <CardContent className="flex flex-col items-center justify-center py-12">
          <Wallet className="mb-4 h-12 w-12 text-muted-foreground" />
          <h3 className="mb-2 text-lg font-semibold">Connect Your Wallet</h3>
          <p className="mb-4 text-center text-muted-foreground">
            Connect your wallet to view your positions, orders, and trading history.
          </p>
          <Button disabled>
            Connect Wallet (Phase 3)
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
