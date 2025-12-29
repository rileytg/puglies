import { useState, useEffect } from "react";
import { useWebSocketStore } from "@/stores/websocket";
import { useAuthStore } from "@/stores/auth";
import { cn } from "@/lib/utils";
import { Circle, Loader2, LogOut, Wallet } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { LoginModal } from "@/components/auth";
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
  const { status: authStatus, checkAuthStatus, logout, balance } = useAuthStore();
  const [loginOpen, setLoginOpen] = useState(false);

  // Check auth status on mount
  useEffect(() => {
    checkAuthStatus();
  }, [checkAuthStatus]);

  const formatAddress = (address: string) => {
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  };

  const formatBalance = (bal: string) => {
    const num = parseFloat(bal) / 1e6; // USDC has 6 decimals
    return num.toLocaleString("en-US", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 2,
    });
  };

  return (
    <>
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

          {/* Auth / User menu */}
          {authStatus.isAuthenticated ? (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size="sm" className="gap-2">
                  <Wallet className="h-4 w-4" />
                  <span className="font-mono text-xs">
                    {formatAddress(authStatus.address || "")}
                  </span>
                  {balance && (
                    <span className="text-muted-foreground">
                      {formatBalance(balance.balance)}
                    </span>
                  )}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem className="font-mono text-xs">
                  {authStatus.address}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem onClick={() => logout()} className="text-destructive">
                  <LogOut className="mr-2 h-4 w-4" />
                  Disconnect
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          ) : (
            <Button variant="outline" size="sm" onClick={() => setLoginOpen(true)}>
              <Wallet className="mr-2 h-4 w-4" />
              Connect
            </Button>
          )}
        </div>
      </header>

      <LoginModal open={loginOpen} onOpenChange={setLoginOpen} />
    </>
  );
}
