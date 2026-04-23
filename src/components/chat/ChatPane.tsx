import { useEffect, useRef } from "react";

import { ChatInput } from "@/components/chat/ChatInput";
import type { Message } from "@/types/message";
import { cn } from "@/lib/utils";

type Props = {
  messages: Message[];
  disabled?: boolean;
  disabledReason?: string;
  onSend: (text: string) => void;
};

export function ChatPane({
  messages,
  disabled,
  disabledReason,
  onSend,
}: Props) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages.length]);

  return (
    <div className="flex h-full flex-col bg-background">
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto px-4 py-4"
      >
        {messages.length === 0 ? (
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
          </div>
        )}
      </div>
      <ChatInput
        disabled={disabled}
        disabledReason={disabledReason}
        onSend={onSend}
      />
    </div>
  );
}

function MessageBubble({ message }: { message: Message }) {
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
