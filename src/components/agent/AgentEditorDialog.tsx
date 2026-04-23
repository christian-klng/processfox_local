import { open } from "@tauri-apps/plugin-dialog";
import { Folder } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { agentApi } from "@/lib/tauri";
import type { Agent } from "@/types/agent";

type Props = {
  open: boolean;
  mode: "create" | "edit";
  agent: Agent | null;
  onClose: () => void;
  onSaved: (agent: Agent) => void;
};

export function AgentEditorDialog({
  open: isOpen,
  mode,
  agent,
  onClose,
  onSaved,
}: Props) {
  const [name, setName] = useState("");
  const [icon, setIcon] = useState("🦊");
  const [folder, setFolder] = useState<string | null>(null);
  const [systemPrompt, setSystemPrompt] = useState("");
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isOpen) return;
    if (mode === "edit" && agent) {
      setName(agent.name);
      setIcon(agent.icon);
      setFolder(agent.folder);
      setSystemPrompt(agent.systemPrompt);
    } else {
      setName("");
      setIcon("🦊");
      setFolder(null);
      setSystemPrompt("");
    }
    setError(null);
  }, [isOpen, mode, agent]);

  async function pickFolder() {
    try {
      const picked = await open({ directory: true, multiple: false });
      if (typeof picked === "string") setFolder(picked);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleSave() {
    setSubmitting(true);
    setError(null);
    try {
      const saved =
        mode === "create"
          ? await agentApi.create({
              name: name.trim(),
              icon,
              folder: folder ?? undefined,
              systemPrompt,
              skills: [],
            })
          : await agentApi.update(agent!.id, {
              name: name.trim(),
              icon,
              folder: folder ?? undefined,
              systemPrompt,
            });
      onSaved(saved);
      onClose();
    } catch (e) {
      // CommandError or plain string
      const msg =
        typeof e === "object" && e && "message" in e
          ? String((e as { message: unknown }).message)
          : String(e);
      setError(msg);
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={(v) => !v && onClose()}>
      <DialogContent className="sm:max-w-[520px]">
        <DialogHeader>
          <DialogTitle>
            {mode === "create" ? "Neuer Agent" : "Agent bearbeiten"}
          </DialogTitle>
        </DialogHeader>

        <div className="flex flex-col gap-4 py-1">
          <div className="flex items-end gap-3">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="agent-icon" className="text-xs">
                Icon
              </Label>
              <Input
                id="agent-icon"
                value={icon}
                onChange={(e) => setIcon(e.target.value)}
                maxLength={2}
                className="w-16 text-center text-lg"
              />
            </div>
            <div className="flex flex-1 flex-col gap-1.5">
              <Label htmlFor="agent-name" className="text-xs">
                Name
              </Label>
              <Input
                id="agent-name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="z. B. Angebots-Assistent"
              />
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-xs">Ordner</Label>
            <div className="flex items-center gap-2">
              <div className="flex-1 truncate rounded-md border border-border bg-background px-3 py-1.5 text-xs text-muted-foreground">
                {folder ?? "Kein Ordner gewählt"}
              </div>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={pickFolder}
                className="gap-2"
              >
                <Folder className="h-3.5 w-3.5" />
                Wählen
              </Button>
            </div>
          </div>

          <div className="flex flex-col gap-1.5">
            <Label htmlFor="agent-prompt" className="text-xs">
              System-Prompt
            </Label>
            <Textarea
              id="agent-prompt"
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              rows={4}
              placeholder="Beschreibe, wie der Agent antworten soll …"
              className="resize-none"
            />
          </div>

          <div className="flex flex-col gap-1.5">
            <Label className="text-xs">Skills</Label>
            <div className="rounded-md border border-dashed border-border bg-muted/40 px-3 py-2 text-xs text-muted-foreground">
              Skills erscheinen hier ab Phase 3.
            </div>
          </div>

          {error && (
            <div className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
              {error}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="ghost" onClick={onClose} disabled={submitting}>
            Abbrechen
          </Button>
          <Button
            onClick={handleSave}
            disabled={submitting || name.trim().length === 0}
          >
            {mode === "create" ? "Anlegen" : "Speichern"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
