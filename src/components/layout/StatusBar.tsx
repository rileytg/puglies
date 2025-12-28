import { useWebSocketStore } from "@/stores/websocket";

export function StatusBar() {
  const { lastUpdate } = useWebSocketStore();

  const formatLastUpdate = () => {
    if (!lastUpdate) return "Never";
    const seconds = Math.floor((Date.now() - lastUpdate.getTime()) / 1000);
    if (seconds < 60) return `${seconds}s ago`;
    const minutes = Math.floor(seconds / 60);
    return `${minutes}m ago`;
  };

  return (
    <footer className="flex h-6 items-center justify-between border-t border-border bg-card px-4 text-xs text-muted-foreground">
      <div>PLGUI v0.1.0</div>
      <div className="flex items-center gap-4">
        <span>Last update: {formatLastUpdate()}</span>
      </div>
    </footer>
  );
}
