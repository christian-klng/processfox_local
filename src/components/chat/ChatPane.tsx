import { useEffect, useRef } from "react";
import { AlertCircle, Loader2, Square, X } from "lucide-react";

import { ChatInput } from "@/components/chat/ChatInput";
import { ReasoningChip } from "@/components/chat/ReasoningChip";
import { ToolCallChip } from "@/components/chat/ToolCallChip";
import { Button } from "@/components/ui/button";
import type { PendingToolCall } from "@/hooks/useAgentChat";
import type { ChatMessage } from "@/types/chat";

type Props = {
  messages: ChatMessage[];
  streamingText: string | null;
  streamingReasoning: string | null;
  pendingTools: PendingToolCall[];
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
  streamingReasoning,
  pendingTools,
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
  }, [messages.length, streamingText, pendingTools.length]);

  const showEmpty =
    messages.length === 0 &&
    streamingText === null &&
    !sending &&
    pendingTools.length === 0;

  // Filter out "tool" messages from display — they're implementation detail;
  // the relevant info lives on the preceding assistant message's tool_calls.
  const visibleMessages = messages.filter((m) => m.role !== "tool");

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
            {visibleMessages.map((m) => (
              <MessageBlock
                key={m.id}
                message={m}
                toolResults={findToolResults(m, messages)}
              />
            ))}

            {streamingReasoning !== null &&
              streamingReasoning.length > 0 && (
                <ReasoningChip text={streamingReasoning} streaming />
              )}

            {pendingTools.length > 0 && (
              <div className="flex flex-col gap-1.5">
                {pendingTools.map((t) => (
                  <ToolCallChip
                    key={t.id}
                    name={t.name}
                    status={t.status}
                    arguments={t.arguments}
                    result={t.content}
                  />
                ))}
              </div>
            )}

            {streamingText !== null && streamingText.length > 0 && (
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
            {pendingTools.some((t) => t.status === "running")
              ? "führt Tool aus …"
              : "generiert …"}
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

/** Find persisted tool results for an assistant message's tool calls by
 *  scanning subsequent tool-role messages in the same history. */
function findToolResults(
  message: ChatMessage,
  all: ChatMessage[],
): Record<string, { content: string; isError: boolean }> {
  const results: Record<string, { content: string; isError: boolean }> = {};
  if (!message.toolCalls || message.toolCalls.length === 0) return results;
  const idx = all.findIndex((m) => m.id === message.id);
  if (idx < 0) return results;
  for (const later of all.slice(idx + 1)) {
    if (later.role !== "tool") break;
    for (const tr of later.toolResults ?? []) {
      results[tr.toolUseId] = { content: tr.content, isError: tr.isError };
    }
  }
  return results;
}

function MessageBlock({
  message,
  toolResults,
}: {
  message: ChatMessage;
  toolResults: Record<string, { content: string; isError: boolean }>;
}) {
  const isUser = message.role === "user";
  const hasToolCalls = (message.toolCalls?.length ?? 0) > 0;
  const hasText = message.content.trim().length > 0;

  if (isUser) {
    return (
      <div className="flex justify-end">
        <div className="max-w-[85%] whitespace-pre-wrap rounded-md bg-primary/10 px-3 py-2 text-sm text-foreground">
          {message.content}
        </div>
      </div>
    );
  }

  // Assistant message: render reasoning chip + tool chips, then the text.
  const reasoning = message.reasoning?.trim();
  return (
    <div className="flex flex-col gap-1.5">
      {reasoning && reasoning.length > 0 && (
        <ReasoningChip text={reasoning} />
      )}
      {hasToolCalls && (
        <div className="flex flex-col gap-1">
          {message.toolCalls!.map((tc) => {
            const res = toolResults[tc.id];
            const status = res ? (res.isError ? "error" : "done") : "done";
            return (
              <ToolCallChip
                key={tc.id}
                name={tc.name}
                status={status}
                arguments={tc.arguments}
                result={res?.content}
              />
            );
          })}
        </div>
      )}
      {hasText && (
        <div className="flex justify-start">
          <div className="max-w-[85%] whitespace-pre-wrap rounded-md bg-muted px-3 py-2 text-sm text-foreground">
            {message.content}
          </div>
        </div>
      )}
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
