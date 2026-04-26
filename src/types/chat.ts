export type ChatMessageRole = "user" | "assistant" | "system" | "tool";

export type HitlPreview =
  | {
      kind: "appendToFile";
      path: string;
      content: string;
      createsFile: boolean;
      /** Last few lines of the existing file, so the reviewer can spot a
       *  format mismatch before approving. Absent for new files. */
      existingTail?: string;
    }
  | {
      kind: "writeDocx";
      path: string;
      blockCount: number;
      previewText: string;
      createsFile: boolean;
    }
  | {
      kind: "appendToDocx";
      path: string;
      blockCount: number;
      previewText: string;
      createsFile: boolean;
      existingTail?: string;
    }
  | {
      kind: "rewriteFile";
      path: string;
      before: string;
      after: string;
      createsFile: boolean;
    };

export type HitlDecision =
  | { kind: "approve" }
  | { kind: "reject"; reason?: string };

export interface PendingHitl {
  hitlId: string;
  toolCallId: string;
  toolName: string;
  preview: HitlPreview;
}

export interface ToolCall {
  id: string;
  name: string;
  arguments: unknown;
}

export interface ToolResult {
  toolUseId: string;
  content: string;
  isError: boolean;
}

export interface ChatMessage {
  id: string;
  role: ChatMessageRole;
  content: string;
  createdAt: string;
  toolCalls?: ToolCall[];
  toolResults?: ToolResult[];
  /** Chain-of-thought / reasoning extracted from the model's output. */
  reasoning?: string;
}

export interface RunStarted {
  runId: string;
  assistantMessageId: string;
}

export type RunEvent =
  | { type: "delta"; text: string }
  | { type: "reasoningDelta"; text: string }
  | {
      type: "toolCallStarted";
      id: string;
      name: string;
      arguments: unknown;
    }
  | {
      type: "toolCallCompleted";
      id: string;
      content: string;
      isError: boolean;
    }
  | {
      type: "hitlRequest";
      hitlId: string;
      toolCallId: string;
      toolName: string;
      preview: HitlPreview;
    }
  | {
      type: "hitlResolved";
      hitlId: string;
      decision: HitlDecision;
    }
  | {
      type: "finish";
      reason: "stop" | "max_tokens" | "cancelled" | "error" | "tool_use";
      message: ChatMessage;
    }
  | { type: "error"; code: string; message: string };
