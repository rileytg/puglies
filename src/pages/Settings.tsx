import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

export function Settings() {
  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold">Settings</h2>

      <Card>
        <CardHeader>
          <CardTitle>Appearance</CardTitle>
          <CardDescription>Customize the look and feel of the app</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">
            Theme settings will be available in Phase 5.
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Trading Preferences</CardTitle>
          <CardDescription>Configure default trading behavior</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">
            Trading preferences will be available after authentication (Phase 4).
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>About</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Version</span>
            <span>0.1.0</span>
          </div>
          <Separator />
          <div className="flex justify-between">
            <span className="text-muted-foreground">Built with</span>
            <span>Tauri + React</span>
          </div>
          <Separator />
          <div className="flex justify-between">
            <span className="text-muted-foreground">Platform</span>
            <span>Polymarket</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
