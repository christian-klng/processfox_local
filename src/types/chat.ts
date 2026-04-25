export type ChatMessageRole = "user" | "assistant" | "system" | "tool";

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
}

export interface RunStarted {
  runId: string;
  assistantMessageId: string;
}

export type RunEvent =
  | { type: "delta"; text: string }
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
      type: "finish";
      reason: "stop" | "max_tokens" | "cancelled" | "error" | "tool_use";
      message: ChatMessage;
    }
  | { type: "error"; code: string; message: string };
