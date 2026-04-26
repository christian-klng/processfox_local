import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { ArrowUp } from "lucide-react";

import { Button } from "@/components/ui/button";

type Props = {
  disabled?: boolean;
  disabledReason?: string;
  onSend: (text: string) => void;
  /** Bump `token` to set the input value externally (e.g. starter chips).
   *  We watch the token rather than the text so the same prompt can be
   *  applied twice in a row. */
  prefill?: { text: string; token: number };
};

export function ChatInput({ disabled, disabledReason, onSend, prefill }: Props) {
  const [value, setValue] = useState("");
  const ref = useRef<HTMLTextAreaElement>(null);
  const lastTokenRef = useRef<number | null>(null);

  useEffect(() => {
    if (!prefill) return;
    if (lastTokenRef.current === prefill.token) return;
    lastTokenRef.current = prefill.token;
    setValue(prefill.text);
    ref.current?.focus();
  }, [prefill]);

  function handleSend() {
    const trimmed = value.trim();
    if (!trimmed || disabled) return;
    onSend(trimmed);
    setValue("");
  }

  function handleKey(e: KeyboardEvent<HTMLTextAreaElement>) {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      handleSend();
    }
  }

  return (
    <div className="border-t border-border bg-surface p-3">
      <div className="relative rounded-md border border-border bg-background focus-within:border-ring focus-within:ring-1 focus-within:ring-ring">
        <textarea
          ref={ref}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKey}
          disabled={disabled}
          placeholder={
            disabled
              ? (disabledReason ?? "Chat ist deaktiviert.")
              : "Schreib eine Nachricht …  (⌘/Ctrl + Enter zum Senden)"
          }
          rows={3}
          className="block w-full resize-none rounded-md bg-transparent px-3 py-2 pr-10 text-sm placeholder:text-muted-foreground focus:outline-none disabled:cursor-not-allowed disabled:opacity-60"
        />
        <Button
          size="icon"
          className="absolute bottom-1.5 right-1.5 h-7 w-7"
          onClick={handleSend}
          disabled={disabled || value.trim().length === 0}
          title="Senden (⌘/Ctrl + Enter)"
        >
          <ArrowUp className="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>
  );
}
