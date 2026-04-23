import { useTheme, type Theme } from "@/components/theme-provider";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

type Props = {
  open: boolean;
  onClose: () => void;
};

const THEME_OPTIONS: { value: Theme; label: string }[] = [
  { value: "system", label: "System" },
  { value: "light", label: "Hell" },
  { value: "dark", label: "Dunkel" },
];

export function SettingsDialog({ open, onClose }: Props) {
  const { theme, setTheme } = useTheme();

  return (
    <Dialog open={open} onOpenChange={(v) => !v && onClose()}>
      <DialogContent className="sm:max-w-[640px]">
        <DialogHeader>
          <DialogTitle>Einstellungen</DialogTitle>
        </DialogHeader>

        <Tabs defaultValue="models">
          <TabsList className="w-full justify-start">
            <TabsTrigger value="models">Modelle</TabsTrigger>
            <TabsTrigger value="cloud">Cloud-APIs</TabsTrigger>
            <TabsTrigger value="appearance">Darstellung</TabsTrigger>
            <TabsTrigger value="about">Über</TabsTrigger>
          </TabsList>

          <TabsContent
            value="models"
            className="py-4 text-xs text-muted-foreground"
          >
            Modell-Download und -Verwaltung folgen in Phase 2.
          </TabsContent>

          <TabsContent
            value="cloud"
            className="py-4 text-xs text-muted-foreground"
          >
            API-Keys für Anthropic, OpenAI und OpenRouter folgen in Phase 2.
          </TabsContent>

          <TabsContent value="appearance" className="py-4">
            <div className="flex flex-col gap-3">
              <div className="text-sm font-medium">Theme</div>
              <div className="flex gap-2">
                {THEME_OPTIONS.map((opt) => (
                  <button
                    key={opt.value}
                    onClick={() => setTheme(opt.value)}
                    className={`rounded-md border px-3 py-1.5 text-xs transition-colors ${
                      theme === opt.value
                        ? "border-primary bg-primary/10 text-foreground"
                        : "border-border bg-background text-muted-foreground hover:bg-accent"
                    }`}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            </div>
          </TabsContent>

          <TabsContent value="about" className="py-4">
            <div className="flex flex-col gap-1 text-xs">
              <div className="text-sm font-medium">ProcessFox</div>
              <div className="text-muted-foreground">Version 0.1.0 (Phase 1)</div>
              <div className="text-muted-foreground">
                Lokale KI-Agenten für Einsteiger.
              </div>
            </div>
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}
