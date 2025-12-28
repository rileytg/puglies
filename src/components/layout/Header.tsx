import { useWebSocketStore } from "@/stores/websocket";
import { cn } from "@/lib/utils";
import { Circle, User, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { ConnectionStateValue } from "@/lib/types";

function ConnectionIndicator({
  label,
  state,
}: {
  label: string;
  state: ConnectionStateValue;
}) {
  const getColor = () => {
    switch (state) {
      case "connected":
        return "text-green-500";
      case "connecting":
      case "reconnecting":
        return "text-yellow-500";
      case "failed":
        return "text-red-500";
      default:
        return "text-muted-foreground";
    }
  };

  const isAnimating = state === "connecting" || state === "reconnecting";

  return (
    <div className="flex items-center gap-1.5">
      {isAnimating ? (
        <Loader2 className={cn("h-3 w-3 animate-spin", getColor())} />
      ) : (
        <Circle className={cn("h-2 w-2 fill-current", getColor())} />
      )}
      <span className="text-xs text-muted-foreground">{label}</span>
    </div>
  );
}

export function Header() {
  const { status } = useWebSocketStore();

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
        <div className="flex items-center gap-3 text-sm">
          <ConnectionIndicator label="RTDS" state={status.rtds} />
          <ConnectionIndicator label="CLOB" state={status.clob} />
        </div>

        {/* User button (placeholder for auth) */}
        <Button variant="ghost" size="icon" className="rounded-full">
          <User className="h-5 w-5" />
        </Button>
      </div>
    </header>
  );
}
