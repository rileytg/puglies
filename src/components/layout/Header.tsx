import { useWebSocketStore } from "@/stores/websocket";
import { cn } from "@/lib/utils";
import { Circle, User } from "lucide-react";
import { Button } from "@/components/ui/button";

export function Header() {
  const { status } = useWebSocketStore();

  const isConnected = status.clob === "connected" || status.rtds === "connected";

  return (
    <header className="flex h-14 items-center justify-between border-b border-border bg-card px-4">
      {/* Title */}
      <div className="flex items-center gap-3">
        <h1 className="text-lg font-semibold">PLGUI</h1>
        <span className="text-sm text-muted-foreground">Polymarket Desktop</span>
      </div>

      {/* Right side */}
      <div className="flex items-center gap-4">
        {/* Connection status */}
        <div className="flex items-center gap-2 text-sm">
          <Circle
            className={cn(
              "h-2 w-2 fill-current",
              isConnected ? "text-green" : "text-muted-foreground"
            )}
          />
          <span className="text-muted-foreground">
            {isConnected ? "Connected" : "Disconnected"}
          </span>
        </div>

        {/* User button (placeholder for auth) */}
        <Button variant="ghost" size="icon" className="rounded-full">
          <User className="h-5 w-5" />
        </Button>
      </div>
    </header>
  );
}
