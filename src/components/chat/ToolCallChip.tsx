import { useState } from "react";
import { AlertTriangle, Check, ChevronRight, Loader2 } from "lucide-react";

import { iconForTool } from "@/lib/toolIcons";
import { cn } from "@/lib/utils";

export type ToolChipStatus = "running" | "done" | "error";

type Props = {
  name: string;
  status: ToolChipStatus;
  arguments?: unknown;
  result?: string;
};

export function ToolCallChip({ name, status, arguments: args, result }: Props) {
  const [expanded, setExpanded] = useState(false);

  const argsText = (() => {
    if (args === undefined || args === null) return null;
    if (typeof args === "string") return args;
    try {
      return JSON.stringify(args, null, 2);
    } catch {
      return String(args);
    }
  })();

  const canExpand = Boolean(argsText) || Boolean(result);
  const ToolIcon = iconForTool(name);

  return (
    <div
      className={cn(
        "flex flex-col gap-1 rounded-md border px-2.5 py-1.5 text-xs",
        status === "running" &&
          "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300",
        status === "done" &&
          "border-emerald-500/30 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
        status === "error" &&
          "border-destructive/40 bg-destructive/10 text-destructive",
      )}
    >
      <button
        type="button"
        onClick={() => canExpand && setExpanded((v) => !v)}
        disabled={!canExpand}
        className="flex w-full items-center gap-1.5 text-left"
      >
        {status === "running" ? (
          <Loader2 className="h-3 w-3 shrink-0 animate-spin" />
        ) : status === "done" ? (
          <Check className="h-3 w-3 shrink-0" />
        ) : (
          <AlertTriangle className="h-3 w-3 shrink-0" />
        )}
        <ToolIcon className="h-3 w-3 shrink-0 opacity-60" />
        <span className="font-mono">{name}</span>
        {canExpand && (
          <ChevronRight
            className={cn(
              "ml-auto h-3 w-3 shrink-0 opacity-60 transition-transform",
              expanded && "rotate-90",
            )}
          />
        )}
      </button>

      {expanded && (
        <div className="mt-1 flex flex-col gap-1.5 text-[11px]">
          {argsText && (
            <div>
              <div className="opacity-60">Arguments</div>
              <pre className="mt-0.5 max-h-32 overflow-auto rounded-sm bg-background/60 p-1.5 font-mono whitespace-pre-wrap">
                {argsText}
              </pre>
            </div>
          )}
          {result && (
            <div>
              <div className="opacity-60">
                {status === "error" ? "Error" : "Result"}
              </div>
              <pre className="mt-0.5 max-h-40 overflow-auto rounded-sm bg-background/60 p-1.5 font-mono whitespace-pre-wrap">
                {result.length > 2000 ? `${result.slice(0, 2000)}\n…` : result}
              </pre>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
