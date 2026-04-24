import { useEffect, useRef } from "react";
import { AlertCircle, Loader2, Square, X } from "lucide-react";

import { ChatInput } from "@/components/chat/ChatInput";
import { Button } from "@/components/ui/button";
import type { ChatMessage } from "@/types/chat";
import { cn } from "@/lib/utils";

type Props = {
  messages: ChatMessage[];
  streamingText: string | null;
  sending: boolean;
  error: string | null;
  disabled?: boolean;
  disabledReason?: string;
  onSend: (text: string) => void;
  onCancel: () => void;
  onDismissError: () => void;
  onOpenSettings?: () => void;
};

export function ChatPane({
  messages,
  streamingText,
  sending,
  error,
  disabled,
  disabledReason,
  onSend,
  onCancel,
  onDismissError,
  onOpenSettings,
}: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages.length, streamingText]);

  const showEmpty = messages.length === 0 && streamingText === null && !sending;

  return (
    <div className="flex h-full flex-col bg-background">
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-4 py-4">
        {showEmpty ? (
          <div className="flex h-full flex-col items-center justify-center gap-1 text-center">
            <div className="text-sm font-medium">Leerer Chat</div>
            <div className="text-xs text-muted-foreground">
              Probier mal: „Fasse mir alle PDFs zusammen."
            </div>
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            {messages.map((m) => (
              <MessageBubble key={m.id} message={m} />
            ))}
            {streamingText !== null && (
              <StreamingBubble text={streamingText} />
            )}
          </div>
        )}
      </div>

      {error && (
        <div className="flex items-start gap-2 border-t border-destructive/30 bg-destructive/10 px-4 py-2 text-xs text-destructive">
          <div className="flex-1">{error}</div>
          <button
            onClick={onDismissError}
            className="text-destructive/70 hover:text-destructive"
            title="Schließen"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        </div>
      )}

      {sending && (
        <div className="flex items-center justify-between gap-2 border-t border-border bg-muted/40 px-4 py-2 text-xs text-muted-foreground">
          <div className="flex items-center gap-2">
            <Loader2 className="h-3.5 w-3.5 animate-spin" />
            generiert …
          </div>
          <Button size="sm" variant="outline" onClick={onCancel} className="gap-1.5">
            <Square className="h-3 w-3" />
            Stopp
          </Button>
        </div>
      )}

      {disabled && disabledReason && !sending && (
        <div className="flex items-start gap-2 border-t border-amber-500/30 bg-amber-500/10 px-4 py-2 text-xs text-amber-700 dark:text-amber-300">
          <AlertCircle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
          <div className="flex-1">{disabledReason}</div>
          {onOpenSettings && (
            <button
              onClick={onOpenSettings}
              className="shrink-0 rounded-sm border border-amber-500/40 bg-amber-500/10 px-2 py-0.5 text-[11px] hover:bg-amber-500/20"
            >
              Einstellungen öffnen
            </button>
          )}
        </div>
      )}

      <ChatInput
        disabled={disabled || sending}
        disabledReason={disabledReason}
        onSend={onSend}
      />
    </div>
  );
}

function MessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === "user";
  return (
    <div className={cn("flex", isUser ? "justify-end" : "justify-start")}>
      <div
        className={cn(
          "max-w-[85%] whitespace-pre-wrap rounded-md px-3 py-2 text-sm",
          isUser
            ? "bg-primary/10 text-foreground"
            : "bg-muted text-foreground",
        )}
      >
        {message.content}
      </div>
    </div>
  );
}

function StreamingBubble({ text }: { text: string }) {
  return (
    <div className="flex justify-start">
      <div className="max-w-[85%] whitespace-pre-wrap rounded-md bg-muted px-3 py-2 text-sm text-foreground">
        {text}
        <span className="ml-0.5 inline-block h-3 w-1 translate-y-0.5 animate-pulse bg-foreground/60" />
      </div>
    </div>
  );
}
