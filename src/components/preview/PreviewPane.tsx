import { FileText, X } from "lucide-react";

import { Button } from "@/components/ui/button";

type Props = {
  fileName: string | null;
  filePath: string | null;
  onClose: () => void;
};

export function PreviewPane({ fileName, filePath, onClose }: Props) {
  if (!fileName) {
    return (
      <div className="flex h-full items-center justify-center p-6 text-xs text-muted-foreground">
        Wähle links eine Datei, um ihre Vorschau zu öffnen.
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between gap-2 border-b border-border bg-surface px-3 py-2">
        <div className="flex min-w-0 items-center gap-2">
          <FileText className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
          <span className="truncate text-sm font-medium">{fileName}</span>
        </div>
        <Button
          variant="ghost"
          size="icon"
          className="h-7 w-7"
          onClick={onClose}
          title="Vorschau schließen"
        >
          <X className="h-3.5 w-3.5" />
        </Button>
      </div>
      <div className="flex flex-1 flex-col items-center justify-center gap-2 p-6 text-center">
        <div className="text-sm font-medium">{fileName}</div>
        <div className="max-w-md truncate text-xs text-muted-foreground">
          {filePath}
        </div>
        <div className="mt-2 text-xs text-muted-foreground">
          Vorschau-Rendering folgt in Phase 3.
        </div>
      </div>
    </div>
  );
}
